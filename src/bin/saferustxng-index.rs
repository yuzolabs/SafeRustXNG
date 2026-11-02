//! Query and inspect an existing local index without making network requests.
//!
//!   saferustxng-index search --index ./index-data "tokio runtime"
//!   saferustxng-index stats --index ./index-data

use std::path::PathBuf;
use std::time::Instant;

use saferustxng::localindex::LocalIndex;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run_index_command(&args) {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run_index_command(args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or("");
    if !matches!(command, "search" | "stats") {
        return Err("usage: saferustxng-index <search|stats> [--index PATH] [--limit N] [QUERY]; HTML crawling is not supported".into());
    }
    let options = IndexOptions::parse(&args[1..])?;
    match command {
        "search" => search(&options),
        "stats" => stats(&options),
        _ => unreachable!(),
    }
}

struct IndexOptions {
    index: PathBuf,
    limit: usize,
    query: Vec<String>,
}

impl IndexOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            index: PathBuf::from("index-data"),
            limit: 10,
            query: Vec::new(),
        };
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--index" => {
                    options.index = args.next().ok_or("--index needs a path")?.into();
                }
                "--limit" => {
                    options.limit = args
                        .next()
                        .ok_or("--limit needs a number")?
                        .parse()
                        .map_err(|_| "--limit needs a non-negative integer")?;
                }
                other if other.starts_with("--") => {
                    return Err(format!("unsupported index option: {other}"));
                }
                other => options.query.push(other.to_string()),
            }
        }
        Ok(options)
    }
}

fn search(options: &IndexOptions) -> Result<(), String> {
    let query = options.query.join(" ");
    if query.is_empty() {
        return Err("no query given".to_string());
    }
    let index = LocalIndex::open(&options.index)?;
    let started = Instant::now();
    let hits = index.search(&query, options.limit)?;
    println!(
        "{} hits in {:.1} ms (index has {} documents)",
        hits.len(),
        started.elapsed().as_secs_f64() * 1000.0,
        index.document_count()
    );
    for (rank, hit) in hits.iter().enumerate() {
        println!("{:>3}. {:.3}  {}", rank + 1, hit.score, hit.url);
        if !hit.title.is_empty() {
            println!("      {}", hit.title);
        }
    }
    Ok(())
}

fn stats(options: &IndexOptions) -> Result<(), String> {
    let index = LocalIndex::open(&options.index)?;
    println!("documents: {}", index.document_count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_crawl_command_is_rejected_before_opening_an_index() {
        let args = vec!["crawl".into(), "--seeds".into(), "seeds.txt".into()];
        assert!(
            run_index_command(&args)
                .unwrap_err()
                .contains("HTML crawling is not supported")
        );
    }

    #[test]
    fn crawler_options_are_not_accepted() {
        for option in [
            "--seeds",
            "--out",
            "--sitemaps",
            "--max-pages",
            "--max-depth",
            "--workers",
            "--delay-ms",
        ] {
            assert!(IndexOptions::parse(&[option.into()]).is_err());
        }
    }

    #[test]
    fn local_search_options_are_preserved() {
        let args = ["--index", "local-index", "--limit", "5", "rust", "async"].map(str::to_string);
        let options = IndexOptions::parse(&args).unwrap();
        assert_eq!(options.index, PathBuf::from("local-index"));
        assert_eq!(options.limit, 5);
        assert_eq!(options.query, ["rust", "async"]);
        assert!(IndexOptions::parse(&["--index".into()]).is_err());
        assert!(IndexOptions::parse(&["--limit".into(), "bad".into()]).is_err());
    }
}
