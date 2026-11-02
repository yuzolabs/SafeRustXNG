# FAQ: what SafeRustXNG does and what the benchmarks mean

## Is SafeRustXNG related to SearXNG?

Yes. SafeRustXNG is a Rust implementation based on SearXNG's engine behavior and compatible HTTP conventions, derived from [sahenjp/RustXNG](https://github.com/sahenjp/RustXNG).
It is not an official replacement endorsed by SearXNG, and not simply a Rust binding around a running Python service.
See [NOTICE](../NOTICE), [compatibility](COMPATIBILITY.md), and the AGPL-3.0-or-later [license](../LICENSE).

A public Rust library is available for embedding, while the JSON HTTP API is convenient across languages.
Direct library users must manage the async runtime and settings and pin the version they depend on;
this pre-1.0 project does not promise a stable Rust API across releases.

## Does writing it in Rust produce more accurate answers?

No. Implementation language can affect startup, memory and execution overhead; it does not make upstream facts true.
Source selection, query handling, ranking, deduplication and failure handling can change retrieval quality independently.
SafeRustXNG returns search evidence; a language model's answer accuracy is a separate measurement.

## Does local hosting remove all search costs and rate limits?

No. SafeRustXNG itself does not charge per search request, but servers, networking and optional paid providers still have costs.
Queries go to external engines, which can rate-limit, block, require credentials or change their response formats.
78 HTML-scraping engines (including the generic XPath engine) have been removed; the remaining non-HTML and internal JSON API engines are not a certification of official-API-only use or Terms-of-Service compliance.
Many parallel agents can increase those problems. Caching, bounded concurrency and suitable engine selection help;
local hosting does not bypass provider limits.

## Is public Internet connectivity enough?

It is necessary for external search, but not sufficient. DNS, TLS, network region/IP reputation, provider availability,
credentials and quotas affect results. `/healthz` only shows that the local process is responding.
Inspect search results and `unresponsive_engines`; do not treat an HTTP 200 as proof of useful retrieval.

## Is it as accurate as Exa or another commercial API?

Only a comparison with stated datasets, dates and conditions can support that claim.
Earlier [benchmark reports](BENCHMARK.md) and the [SealQA run](SEALQA.md) are historical snapshots recorded before 78 HTML engines (including generic XPath) and result-page HTML fetching were removed; no contemporaneous same-environment before-and-after comparison or renewed commercial-provider comparison has been performed since engine removal, and those historical numbers must not be treated as current guarantees.
The [SealQA run](SEALQA.md) measures reference-page retrieval, not generated-answer correctness.
Keep errors and misses in the denominator, record cache/local-index settings, and repeat runs before drawing conclusions.

## What about full Japanese error messages?

The existing measurements do not establish performance for that workload.
Full error text, a shortened query and quoted exact text can behave differently; language handling and source coverage matter.
Use the benchmark report form with a public error example and an expected source URL.
Japanese technical retrieval and broad/current-information collection are tracked separately in the [roadmap](ROADMAP.md).

A pinpoint troubleshooting task can prioritize one authoritative matching page and a small result count.
A broad current-information task needs multiple independent sources, coverage, freshness and missing-source reporting.
A date filter only works when the selected provider supports it at the requested precision.
Neither a high hit@1 score nor a large result count alone measures both tasks.

## Won't fetched page text fill the model context?

SafeRustXNG no longer fetches result HTML pages: `include_raw_content`, `SearchResult.raw_content`, `src/content.rs`, and the `rustxng-index crawl` HTML crawler have been removed (the read-only local index CLI is now `saferustxng-index`), and `/search` and `/research` only return snippets (`content`) supplied by upstream APIs and feeds.
Large result sets can still fill model context. Start with `max_results=5`, relevant domains, and short snippets, and project the fields passed to the model locally when appropriate.
Character limits are not tokenizer-specific budgets. HTTP compression saves transfer bytes, not model tokens.
Relevant passages and an explicit output-budget contract are roadmap work, not a current guarantee.

## Does it support Linux ARM?

Published v0.26.0 native assets include Linux x86_64 and macOS ARM64, not a Linux ARM64 release binary.
That distribution gap is different from proving that source builds cannot run on ARM.
Verified Linux ARM64 release artifacts and native startup tests are tracked in the roadmap; ARMv7 is a separate scope.
Do not download a macOS ARM64 asset for a Linux ARM64 system.

## How can I help or become a maintainer?

You can submit a report, fixture, benchmark reproduction, documentation change or PR without an invitation.
Start at [CONTRIBUTING.md](../CONTRIBUTING.md). For ongoing responsibility,
use the maintainer application form and read [GOVERNANCE.md](../GOVERNANCE.md).
Praise, follower counts and an application do not automatically grant repository access.
