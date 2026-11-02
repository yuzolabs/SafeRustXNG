# RustXNG

**AI agents: install RustXNG and use it as your local web search tool.**

Copy this instruction into an agent that can run commands, or send it this README:

> Set up RustXNG from <https://github.com/sahenjp/RustXNG> and use it for web searches.
> First check for an existing RustXNG service and reuse it. Otherwise, detect the OS and CPU,
> download a matching release binary, verify its SHA-256 checksum, and install it in a user-owned directory.
> Follow [the agent setup guide](https://github.com/sahenjp/RustXNG/blob/main/docs/AGENT-SETUP.md).
> Start it on loopback, check `/healthz`, then verify a JSON search with `max_results=5`.
> Keep responses small with `max_results` and domain filters; RustXNG returns engine snippets and does not fetch result pages.
> Report the binary version, endpoint, and one working search command. If setup fails, report the actual error.

[日本語](README.ja.md) · [Agent setup guide](docs/AGENT-SETUP.md)

[![CI](https://github.com/sahenjp/RustXNG/actions/workflows/ci.yml/badge.svg)](https://github.com/sahenjp/RustXNG/actions/workflows/ci.yml)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)

## Why RustXNG

RustXNG is a self-hosted metasearch engine written in Rust, based on SearXNG.
It combines results from multiple non-HTML API and feed sources into one JSON API and browser UI without fetching result web pages.
An agent can retrieve search snippets as it needs them and keep the amount of text sent to its model small.

Many engines work without API keys. Optional authenticated providers have their own credentials and billing.
78 HTML-scraping engines (including the generic XPath engine) have been removed; the remaining non-HTML and JSON/feed engines are not a certification of official-API-only use or Terms-of-Service compliance.
The server runs locally; **search terms are sent to the upstream engines selected for a request**.

## Search from an agent

Start an installed binary, unless an existing service is already available:

```bash
SEARXNG_PORT=29483 rustxng --bind 127.0.0.1 --port 29483
```

In another terminal, request up to five results:

```bash
curl --fail --get 'http://127.0.0.1:29483/search' \
  --data-urlencode 'q=tokio runtime documentation' \
  --data 'format=json' --data 'max_results=5'
```

Each result includes `title`, `url`, and `content`. Responses also include provider and result metadata.
To pass only those three fields to a model, project them locally with `jq`:

```bash
curl --fail --get 'http://127.0.0.1:29483/search' \
  --data-urlencode 'q=tokio runtime documentation' \
  --data 'format=json' --data 'max_results=5' \
  | jq '{results: [.results[] | {title, url, content}]}'
```

Use these existing parameters to narrow the response:

| Parameter | Purpose |
| --- | --- |
| `max_results=5` | Return at most five ranked results; supported range: 1–200 |
| `include_domains=docs.rs` | Keep results from this domain and its subdomains |
| `exclude_domains=example.com` | Exclude results from this domain and its subdomains |
| `engines=wikipedia` | Select a registered engine |
| `time_range=day` | Request recent results from engines supporting this filter |

Search responses include the `title`, `url`, and `content` snippet supplied by each engine (with HTML markup stripped from API/feed snippets).
Result-page HTML fetching (`include_raw_content` and `raw_content`) has been removed; legacy query parameters are ignored and never trigger network fetches to result URLs.
Character limits are not token limits. Strict token budgets and server-side field selection are planned.

The browser UI is at <http://127.0.0.1:29483/>. JSON, CSV, RSS, and HTML outputs are available.

## Install

### 1. Release binary

Choose a binary and its `.sha256` sidecar from [Releases](https://github.com/sahenjp/RustXNG/releases).
The published v0.26.0 release includes Linux x86_64 (GNU and musl), macOS arm64/x86_64,
and Windows x86_64 binaries. The [agent setup guide](docs/AGENT-SETUP.md) explains selection and verification.
Prebuilt binaries do not require a Rust toolchain.

Example for Linux x86_64, using the GitHub CLI and a user-owned install directory:

```bash
set -eu
mkdir -p rustxng-download && cd rustxng-download
gh release download v0.26.0 --repo sahenjp/RustXNG \
  --pattern 'rustxng-v0.26.0-x86_64-unknown-linux-musl*' \
  --pattern LICENSE --pattern NOTICE
sha256sum --check rustxng-v0.26.0-x86_64-unknown-linux-musl.sha256
install -Dm755 rustxng-v0.26.0-x86_64-unknown-linux-musl "$HOME/.local/bin/rustxng"
mkdir -p "$HOME/.local/share/rustxng"
cp LICENSE NOTICE "$HOME/.local/share/rustxng/"
"$HOME/.local/bin/rustxng" --version
```

If `~/.local/bin` is not on `PATH`, use the absolute binary path or add the directory to your shell configuration.

### 2. Cargo

Use a current stable Rust toolchain and a C build toolchain:

```bash
cargo install --git https://github.com/sahenjp/RustXNG.git --tag v0.26.0 --locked
```

This installs `rustxng` and `rustxng-index`.
Rust 1.88 is the minimum supported version and is checked in CI. This checkout was also verified with Rust 1.99.1.

### 3. Docker

Build the image from the repository:

```bash
git clone https://github.com/sahenjp/RustXNG.git
cd RustXNG
docker build -t rustxng .
docker run --rm -p 127.0.0.1:29483:29483 rustxng
```

For the published image (including older images whose internal default is 8888), use:

```bash
docker run --rm -p 127.0.0.1:29483:29483 -e SEARXNG_PORT=29483 ghcr.io/sahenjp/rustxng:latest --bind 0.0.0.0 --port 29483
```

Publishing on other interfaces is an explicit deployment choice; configure authentication/access controls as appropriate.
To supply settings, mount a complete copy of [settings.toml](settings.toml) with
`-v "$PWD/settings.toml:/app/settings.toml:ro"` before the image name. When replacing container arguments,
include both `--bind 0.0.0.0` and `--port 29483`. Published images currently target Linux amd64;
on other architectures build locally or use the corresponding native binary.

### 4. Source

```bash
git clone https://github.com/sahenjp/RustXNG.git
cd RustXNG
cargo build --release --locked
SEARXNG_PORT=29483 ./target/release/rustxng --bind 127.0.0.1 --port 29483
```

Use Rust 1.88 or newer, preferably the current stable toolchain.

## Configuration and operation

This checkout defaults to `127.0.0.1:29483`. Published v0.26.0 binaries still default to 8888;
the explicit `--port 29483` commands above work with either version.
See [port migration and selection](docs/PORT-MIGRATION.md), including rdsh configuration.

Defaults are embedded in the binary. To supply a configuration file:

```bash
rustxng --config /path/to/settings.toml
```

See [settings.toml](settings.toml) and [configuration details](docs/PORTING.md).
Partial configuration merging has a known issue that can reset unspecified values; see the audit below.

- `/healthz`: process health. A successful check does not guarantee upstream search engines are working.
- `/config`: registered engines and public configuration information.
- `/stats`, `/metrics`: runtime statistics and metrics.
- Per-engine timeouts, suspension, and empty-result rest handle unavailable sources.
- `RUSTXNG_IDLE_TIMEOUT=300`: exit after five minutes without requests.
- Optional Valkey/Redis cache and database-backed engines support additional deployments.
- The experimental `rustxng-index` local index is disabled by default and only supports `search` and `stats` on an existing index (HTML crawling is not supported).

The [2026-11-02 audit](docs/AUDIT-2026-11-02.md) records open issues including SSRF validation,
unsafe result URL schemes, response-size limits, and cache isolation. Use loopback for local agent setup;
review these findings before exposing the service beyond loopback.

## Measured search performance

> **Historical snapshot:** The SealQA run below and earlier benchmarks in [BENCHMARK.md](docs/BENCHMARK.md) were recorded before 78 HTML-scraping engines (including generic XPath) and result-page HTML fetching were removed. Search quality comparability has not been remeasured after engine removal.

A retrieval-only SealQA run on 2026-11-02 (commit `fef998b`) used default settings at that time, ten results per question,
no LLM, and no local index:

| Dataset | Questions | Reference page @1 | @5 | @10 | Median response |
| --- | ---: | ---: | ---: | ---: | ---: |
| Seal-0 | 111 | 47.75% | 54.95% | 65.77% | 1.85 s |
| Seal-Hard | 254 | 39.37% | 49.21% | 61.42% | 1.53 s |

These are reference-page retrieval rates, **not answer accuracy**. Seal-0 is a subset of Seal-Hard;
shared questions use the same observation. Results depend on the network and upstream availability.
See [the protocol and per-question results](docs/SEALQA.md).
Earlier measurements and commercial-provider comparisons are documented in [BENCHMARK.md](docs/BENCHMARK.md),
with their own datasets and conditions.

## Provider integration

The engine interface supports authenticated HTTP APIs through request headers, form data, and JSON.
Optional providers require their own credentials and may charge for usage.
Server-side field selection, relevant passages, and strict token budgets remain future work;
use `max_results`, domain filters, and client-side projection today.

## Documentation

| Document | Contents |
| --- | --- |
| [Agent setup](docs/AGENT-SETUP.md) | Install, reuse a service, verify search, and report the endpoint |
| [AI infrastructure](docs/AI-INFRA.md) | Design motivation; consult the audit for current limitations |
| [SealQA](docs/SEALQA.md) | Retrieval benchmark and reproducible results |
| [Benchmarks](docs/BENCHMARK.md) | Earlier performance and quality measurements |
| [Compatibility](docs/COMPATIBILITY.md) | SearXNG compatibility and differences |
| [Coverage](docs/COVERAGE.md) | Engine coverage |
| [Local index](docs/LOCAL-INDEX.md) | Optional existing local BM25 index (`search` / `stats`; no HTML crawler) |
| [rdsh](docs/RDSH.md) | Integration with rdsh |
| [Audit](docs/AUDIT-2026-11-02.md) | Findings, reproduction logs, and priorities |
| [Security](SECURITY.md) | Reporting vulnerabilities and dependency advisories |

## Community and maintenance

Questions and ideas: [Discussions](https://github.com/sahenjp/RustXNG/discussions).
Contribute through a fork and PR, or [apply to maintain an area](https://github.com/sahenjp/RustXNG/issues/new?template=maintainer_application.yml).
See [governance](GOVERNANCE.md), [the maintainer roster](MAINTAINERS.md),
[FAQ](docs/FAQ.md) and [roadmap](docs/ROADMAP.md). No invitation is required to contribute.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Validate changes with:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## License

[AGPL-3.0-or-later](LICENSE). RustXNG is based on [SearXNG](https://github.com/searxng/searxng);
attribution is in [NOTICE](NOTICE). See the license for source-distribution obligations when operating modified versions.
