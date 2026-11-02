//! Local full-text index (BM25) for previously stored documents; no web crawler.
//!
//! This is the piece that makes RustXNG independent of upstream engines: a
//! query answered from here costs milliseconds instead of a fan-out.
//!
//! Two tokenisations are indexed for every document without assuming a
//! language: the default tokenizer for space-separated text, and
//! character bigrams for Japanese/Chinese, which have no word delimiters.

use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, Value};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer};
use tantivy::{Index, IndexReader, ReloadPolicy, Term};

/// One search hit.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub url: String,
    pub title: String,
    pub score: f32,
}

/// A name for the CJK tokenizer, registered once per index.
const BIGRAM: &str = "bigram";

/// Fields of the schema, kept together so the searcher can address them.
struct Fields {
    url: Field,
    title: Field,
    body: Field,
    title_ja: Field,
    body_ja: Field,
}

/// An opened local index.
pub struct LocalIndex {
    index: Index,
    reader: IndexReader,
    fields: Fields,
}

#[cfg(test)]
fn build_schema() -> tantivy::schema::Schema {
    use tantivy::schema::{STORED, STRING, Schema, TextFieldIndexing, TextOptions};

    let mut builder = Schema::builder();
    // The plain tokenizer for space-separated languages.
    let plain = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );
    // Bigrams for languages without spaces.
    let ngram = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(BIGRAM)
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );
    builder.add_text_field("url", STRING | STORED);
    builder.add_text_field("title", plain.clone() | STORED);
    builder.add_text_field("body", plain);
    builder.add_text_field("title_ja", ngram.clone() | STORED);
    builder.add_text_field("body_ja", ngram);
    builder.add_text_field("lang", STRING | STORED);
    builder.build()
}

/// True when the text contains Japanese or Chinese characters.
pub fn has_cjk(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x3040..=0x30ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff))
}

impl LocalIndex {
    /// Open the index for reading. Safe to call from any number of processes,
    /// including while another process writes documents.
    pub fn open(path: &Path) -> Result<Self, String> {
        let index = Index::open_in_dir(path)
            .map_err(|error| format!("open existing index at {path:?}: {error}"))?;
        let tokenizer = TextAnalyzer::builder(
            NgramTokenizer::new(2, 2, false).map_err(|error| error.to_string())?,
        )
        .filter(LowerCaser)
        .build();
        index.tokenizers().register(BIGRAM, tokenizer);
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|error| format!("reader: {error}"))?;
        let schema = index.schema();
        let fields = Fields {
            url: schema.get_field("url").map_err(|error| error.to_string())?,
            title: schema
                .get_field("title")
                .map_err(|error| error.to_string())?,
            body: schema
                .get_field("body")
                .map_err(|error| error.to_string())?,
            title_ja: schema
                .get_field("title_ja")
                .map_err(|error| error.to_string())?,
            body_ja: schema
                .get_field("body_ja")
                .map_err(|error| error.to_string())?,
        };
        Ok(Self {
            index,
            reader,
            fields,
        })
    }

    /// Number of indexed documents.
    pub fn document_count(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Rank the indexed pages for a query.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Hit>, String> {
        let searcher = self.reader.searcher();
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        // Space-separated text goes through the standard parser.
        let parser = QueryParser::for_index(&self.index, vec![self.fields.title, self.fields.body]);
        if let Ok(parsed) = parser.parse_query(trimmed) {
            clauses.push((Occur::Should, parsed));
        }
        // CJK text has no spaces: match character bigrams instead.
        if has_cjk(trimmed) {
            let lowered: Vec<char> = trimmed.to_lowercase().chars().collect();
            for pair in lowered.windows(2) {
                if !has_cjk(&pair.iter().collect::<String>()) {
                    continue;
                }
                let gram: String = pair.iter().collect();
                for field in [self.fields.title_ja, self.fields.body_ja] {
                    clauses.push((
                        Occur::Should,
                        Box::new(TermQuery::new(
                            Term::from_field_text(field, &gram),
                            IndexRecordOption::WithFreqs,
                        )),
                    ));
                }
            }
        }
        if clauses.is_empty() {
            return Ok(Vec::new());
        }
        let query = BooleanQuery::new(clauses);
        let top = searcher
            .search(&query, &TopDocs::with_limit(limit).order_by_score())
            .map_err(|error| format!("search: {error}"))?;
        let mut hits = Vec::new();
        for (score, address) in top {
            let document: tantivy::TantivyDocument = searcher
                .doc(address)
                .map_err(|error| format!("doc: {error}"))?;
            let text = |field: Field| {
                document
                    .get_first(field)
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_default()
            };
            hits.push(Hit {
                url: text(self.fields.url),
                title: text(self.fields.title),
                score,
            });
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::doc;
    use tempfile::TempDir;

    fn existing_index(pages: &[(&str, &str, &str)]) -> (TempDir, LocalIndex) {
        let dir = TempDir::new().unwrap();
        let schema = build_schema();
        let index = Index::create_in_dir(dir.path(), schema.clone()).unwrap();
        let tokenizer = TextAnalyzer::builder(NgramTokenizer::new(2, 2, false).unwrap())
            .filter(LowerCaser)
            .build();
        index.tokenizers().register(BIGRAM, tokenizer);
        let mut writer = index.writer(50_000_000).unwrap();
        for &(url, title, body) in pages {
            writer
                .add_document(doc!(
                    schema.get_field("url").unwrap() => url,
                    schema.get_field("title").unwrap() => title,
                    schema.get_field("body").unwrap() => body,
                    schema.get_field("title_ja").unwrap() => title,
                    schema.get_field("body_ja").unwrap() => body,
                    schema.get_field("lang").unwrap() => "",
                ))
                .unwrap();
        }
        writer.commit().unwrap();
        drop(writer);
        let reader = LocalIndex::open(dir.path()).unwrap();
        (dir, reader)
    }

    #[test]
    fn a_reader_does_not_create_the_directory() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("index-data");
        assert!(
            LocalIndex::open(&missing).is_err(),
            "a missing index is an error"
        );
        assert!(!missing.exists(), "the reader must not create it");
    }

    #[test]
    fn a_reader_does_not_create_an_index_in_an_empty_directory() {
        let dir = TempDir::new().unwrap();
        assert!(LocalIndex::open(dir.path()).is_err());
        assert!(!dir.path().join("meta.json").exists());
    }

    #[test]
    fn a_reader_does_not_replace_a_corrupt_index() {
        let dir = TempDir::new().unwrap();
        let metadata = dir.path().join("meta.json");
        std::fs::write(&metadata, "corrupt index metadata").unwrap();
        assert!(LocalIndex::open(dir.path()).is_err());
        assert_eq!(
            std::fs::read_to_string(metadata).unwrap(),
            "corrupt index metadata"
        );
    }

    #[test]
    fn finds_english_pages_in_an_existing_index() {
        let (_dir, index) = existing_index(&[
            (
                "https://tokio.rs/tokio/tutorial",
                "Tutorial",
                "async runtime for rust with tasks and futures",
            ),
            (
                "https://example.com/other",
                "Other",
                "nothing to do with the query",
            ),
        ]);
        assert_eq!(index.document_count(), 2);
        let hits = index.search("async runtime rust", 5).unwrap();
        assert_eq!(hits[0].url, "https://tokio.rs/tokio/tutorial");
    }

    #[test]
    fn finds_japanese_pages_in_an_existing_index() {
        let (_dir, index) = existing_index(&[(
            "https://www.gsi.go.jp/",
            "国土地理院",
            "地図と測量の情報を提供しています",
        )]);
        let hits = index.search("国土地理院 地図", 5).unwrap();
        assert_eq!(hits[0].url, "https://www.gsi.go.jp/");
    }
}
