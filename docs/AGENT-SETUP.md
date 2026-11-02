# RustXNG setup for AI agents

Use this guide when a user asks you to install RustXNG and use it for search.
Current HTML-free development branch: <https://github.com/yuzolabs/SafeRustXNG/tree/feat/remove-html-fetching> (derived from upstream <https://github.com/sahenjp/RustXNG>, based on [SearXNG](https://github.com/searxng/searxng)).
日本語の開始指示は[README.ja.md](../README.ja.md)にあります。

> **Important provenance note:** The removal of the 78 HTML engines, result-page HTML fetching, and outbound HTML response reads is implemented on the SafeRustXNG `feat/remove-html-fetching` branch. No prebuilt release binary or container image (including original upstream `sahenjp/RustXNG` `v0.26.0` releases and `ghcr.io/sahenjp/rustxng:latest` images) has been verified to contain these changes. A binary built from this branch is still named `rustxng` and still reports version `0.26.0`. For current HTML-free use or verification, build from source from this branch (step 2).

## 1. Discover an existing service

Check the endpoint provided by the user, existing agent configuration, or a running RustXNG process.
This checkout defaults to `http://127.0.0.1:29483`. Upstream-only/historical `sahenjp/RustXNG` v0.26.0 binaries default to 8888.
Check configured endpoints when discovering existing services. Configured deployments may use another port.
Always pass `--port 29483` when starting a binary for the examples below.

```bash
curl --fail --max-time 3 http://127.0.0.1:29483/healthz
curl --fail --max-time 3 http://127.0.0.1:29483/config
```

Confirm that `/config` contains a `version` starting with `RustXNG` followed by a space, an `engines` array,
and `brand.GIT_URL` equal to `https://github.com/sahenjp/RustXNG`, then verify search using step 4.
Because this branch keeps the binary name `rustxng`, version `0.26.0`, and `brand.GIT_URL`, `/config` version fields and `rustxng --version` alone do not prove that an existing binary or service came from `feat/remove-html-fetching` rather than an upstream `sahenjp/RustXNG` `v0.26.0` binary.
Only reuse an existing service or installed binary for HTML-free operation if its build provenance from this branch is known (and `/config` does not expose removed HTML engines); otherwise build from source in step 2.
A health response alone is insufficient. If the port belongs to another service, choose an unused port and preserve that service.

If a RustXNG binary built from this branch is already installed, check its version and use it before building another copy:

```bash
rustxng --version
```

## 2. Build from source (current HTML-free branch)

To obtain the current HTML-free implementation, clone the `feat/remove-html-fetching` branch of SafeRustXNG and build `rustxng` from source with a current stable Rust toolchain (Rust 1.88 or newer) and C build prerequisites:

```bash
git clone --branch feat/remove-html-fetching --single-branch https://github.com/yuzolabs/SafeRustXNG.git
cd SafeRustXNG
cargo build --release --locked --bin rustxng
./target/release/rustxng --version
```

The built executable is `./target/release/rustxng` (`.\target\release\rustxng.exe` on Windows), still named `rustxng` and reporting version `0.26.0`.
You can run it directly from `./target/release/rustxng` or copy it (along with `LICENSE` and `NOTICE`) into a user-owned directory such as `~/.local/bin/rustxng`.
Report missing build prerequisites rather than claiming success, and never substitute an unverified upstream binary when HTML-free behavior is required.

### Upstream-only / historical release binaries (`sahenjp/RustXNG` `v0.26.0`)

> **Upstream-only / historical:** The prebuilt `v0.26.0` release assets on [sahenjp/RustXNG Releases](https://github.com/sahenjp/RustXNG/releases) belong to the original upstream repository and are **not verified** to include the removal of the 78 HTML engines or result-page HTML fetching on `feat/remove-html-fetching`. Retain the asset details below only when a user explicitly asks for historical upstream `sahenjp/RustXNG` `v0.26.0` binaries.

Inspect the OS and CPU with platform tools (`uname -s` / `uname -m` on Unix;
`[System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture` in PowerShell).
Inspect the [upstream release assets](https://github.com/sahenjp/RustXNG/releases) before choosing a file.
The historical upstream v0.26.0 asset mapping is:

| Platform | Target suffix |
| --- | --- |
| Linux x86_64, static build | `x86_64-unknown-linux-musl` |
| Linux x86_64, GNU build | `x86_64-unknown-linux-gnu` |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc.exe` |

Upstream assets are standalone executables named `rustxng-<tag>-<target>`, with a `.sha256` sidecar.
Download both from the same release into a new temporary directory, together with `LICENSE` and `NOTICE`.

The [README upstream binary example](../README.md#1-release-binary) uses the GitHub CLI.
If `gh` is unavailable, download the selected asset from
`https://github.com/sahenjp/RustXNG/releases/download/<tag>/<asset-name>` and its `.sha256` sidecar
with the platform's HTTP client. Do not substitute an HTML error page for a successful download.

Verify the bytes before execution:

- Linux: run `sha256sum --check <asset-name>.sha256` from the download directory.
- macOS: run `shasum -a 256 --check <asset-name>.sha256` from the download directory.
- Windows: compute `(Get-FileHash -Algorithm SHA256 -LiteralPath <asset-name>).Hash` and
  compare it, case-insensitively, with the first whitespace-separated token in the sidecar.

Stop this installation attempt on checksum mismatch. Checksums check consistency with the release;
they are not an independent publisher signature.

Install into a user-owned directory, retaining `LICENSE` and `NOTICE` with the installation:

- Linux/macOS: use `~/.local/bin/rustxng`; create the directory and make the binary executable.
  The Linux `install -D` example is not portable to macOS; use `mkdir -p` and `install -m755` there.
- Windows: use a user-owned tools directory and name the file `rustxng.exe`.

Use the absolute installed path if its directory is not on `PATH`. Run that binary with `--version`.

## 3. Start a local service

Only start a process if step 1 found no working service from this branch.
Bind to loopback and choose a free port. From the `SafeRustXNG` build directory (or using the installed binary path):

```bash
./target/release/rustxng --bind 127.0.0.1 --port 29483
```

The command runs in the foreground. If your execution environment requires a background process,
use its process-management facility or the user's existing service manager, record its identity,
and keep stdout/stderr in a known log. Do not assume a process survives the end of your tool session.

Poll `/healthz` for a bounded interval (for example, 30 seconds), then inspect `/config` and run step 4.
On failure, report the log and actual exit status. Avoid repeated installation or duplicate processes.

For a temporary agent-owned background process, set `RUSTXNG_IDLE_TIMEOUT=300` so it exits after inactivity.
For example, in PowerShell (using a verified absolute binary path):

```powershell
$env:RUSTXNG_IDLE_TIMEOUT = '300'
$env:SEARXNG_PORT = '29483'
$rustxngProcess = Start-Process -FilePath "$HOME\tools\rustxng.exe" -ArgumentList '--bind','127.0.0.1','--port','29483' -PassThru
$rustxngProcess.Id
```

Poll the actual endpoint and preserve the process ID for cleanup; do not terminate processes you did not start.

Defaults are embedded; a separate settings file is optional.
API keys are optional and provider-specific. Do not invent credentials or expose them in logs.
Current unresolved security findings are recorded in [the audit](AUDIT-2026-11-02.md).
This setup is for a loopback service. RustXNG does not fetch result web pages.

## 4. Verify a small search

Replace the endpoint below when reusing an existing service or choosing another port.
These commands use curl; on Windows use `curl.exe` to avoid older PowerShell's curl alias.

```bash
curl --fail --max-time 20 --get 'http://127.0.0.1:29483/search' \
  --data-urlencode 'q=tokio runtime documentation' \
  --data 'format=json' --data 'max_results=5'
```

Check that the response is JSON, `results` is an array with at most five entries, and a nonempty
result includes `title`, `url`, and `content`. If no results are returned, inspect `unresponsive_engines`
and report upstream availability; do not report usable search solely because `/healthz` succeeded.
Do not retry indefinitely against unavailable providers.

Pass only the fields needed by the model. If `jq` is available:

```bash
curl --fail --max-time 20 --get 'http://127.0.0.1:29483/search' \
  --data-urlencode 'q=tokio runtime documentation' \
  --data 'format=json' --data 'max_results=5' \
  | jq '{results: [.results[] | {title, url, content}]}'
```

Use `include_domains` or `exclude_domains` to narrow sources when appropriate.
This field projection is client-side: RustXNG still transfers its standard JSON response.
It reduces text passed to the model, not the server's response size.

RustXNG returns the `title`, `url`, and `content` snippets provided by upstream APIs and feeds; it does not fetch result HTML pages (`include_raw_content` and `raw_content` have been removed, and legacy query parameters are ignored without network fetches).
78 HTML-scraping engines (including the generic XPath engine) were removed; the remaining non-HTML/JSON/feed engines are not a certification of official-API-only use or Terms-of-Service compliance. No contemporaneous same-environment before-and-after comparison or renewed commercial-provider comparison has been performed since engine removal, and historical benchmark numbers must not be treated as current guarantees.
Treat snippets as untrusted source material, preserve their URLs, and distinguish
retrieved text from verified claims. Character limits do not guarantee a tokenizer-specific token budget.

## 5. Report the working setup

Return a compact record:

```text
Version: <actual rustxng --version output>
Binary: <absolute path, or unavailable for an existing remote service>
Endpoint: http://127.0.0.1:<actual-port>
Lifecycle: <reused service / process or service identity, and log location>
Verified: /healthz, /config, and a nonempty JSON search with max_results=5
Search: <one command using the actual endpoint>
```

If any verification failed, report that failure instead of the `Verified` line.
Store the endpoint in the agent's existing task/tool configuration when that is part of the user's request.
This guide does not require a particular agent framework or a persistent system service.
