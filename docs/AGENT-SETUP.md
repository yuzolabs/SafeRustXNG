# SafeRustXNG setup for AI agents

Use this guide when a user asks you to install SafeRustXNG and use it for search.
SafeRustXNG source repository (`main` branch): <https://github.com/yuzolabs/SafeRustXNG/tree/main> (derived from upstream <https://github.com/sahenjp/RustXNG>, based on [SearXNG](https://github.com/searxng/searxng)).
日本語の開始指示は[README.ja.md](../README.ja.md)にあります。

> **Important provenance and migration note:** The removal of the 78 HTML engines, result-page HTML fetching, and outbound HTML response reads is provided by the SafeRustXNG fork (<https://github.com/yuzolabs/SafeRustXNG/tree/main>). No prebuilt release binary or container image (including original upstream `sahenjp/RustXNG` `v0.26.0` releases and `ghcr.io/sahenjp/rustxng:latest` images) has been verified to contain these changes. SafeRustXNG renamed the display name and `--version` / `/config` prefix from `RustXNG` to `SafeRustXNG`, the crate and server binary from `rustxng` to `saferustxng`, the read-only local-index binary from `rustxng-index` to `saferustxng-index`, the idle-timeout environment variable from `RUSTXNG_IDLE_TIMEOUT` to `SAFERUSTXNG_IDLE_TIMEOUT`, and release asset names from `rustxng-<tag>-<target>` to `saferustxng-<tag>-<target>` (with version `0.26.0`, default port `29483`, and `SEARXNG_*` settings unchanged; update existing automation accordingly). Because the version number `0.26.0` matches upstream, check the `SafeRustXNG` display name, `/config` `brand.GIT_URL`, and build provenance. For HTML-free use or verification, build from source from SafeRustXNG `main` (step 2).

## 1. Discover an existing service

Check the endpoint provided by the user, existing agent configuration, or a running SafeRustXNG process.
This checkout defaults to `http://127.0.0.1:29483`. Upstream-only/historical `sahenjp/RustXNG` v0.26.0 binaries default to 8888.
Check configured endpoints when discovering existing services. Configured deployments may use another port.
Always pass `--port 29483` when starting a binary for the examples below.

```bash
curl --fail --max-time 3 http://127.0.0.1:29483/healthz
curl --fail --max-time 3 http://127.0.0.1:29483/config
```

Confirm that `/config` contains a `version` starting with `SafeRustXNG` followed by a space, an `engines` array,
and `brand.GIT_URL` equal to `https://github.com/yuzolabs/SafeRustXNG` (`instance_name` defaults to `SafeRustXNG`, though custom instance names are allowed), then verify search using step 4.
Although current SafeRustXNG builds use the `saferustxng` binary name, report `SafeRustXNG 0.26.0`, and return `https://github.com/yuzolabs/SafeRustXNG` in `brand.GIT_URL` (unlike upstream `sahenjp/RustXNG` `v0.26.0`, which reports `RustXNG` and the upstream URL), the version number `0.26.0` is still the same, so an existing service should only be reused when its `SafeRustXNG` identity, `brand.GIT_URL`, and build provenance are verified.
Only reuse an existing service or installed binary for HTML-free operation if its build provenance from SafeRustXNG `main` is known (and `/config` does not expose removed HTML engines); otherwise build from source in step 2.
A health response alone is insufficient. If the port belongs to another service, choose an unused port and preserve that service.

If a `saferustxng` binary built from SafeRustXNG `main` is already installed, check its version and use it before building another copy:

```bash
saferustxng --version
```

## 2. Build from source (SafeRustXNG `main`)

To obtain the HTML-free implementation, clone the `main` branch of SafeRustXNG and build `saferustxng` from source with a current stable Rust toolchain (Rust 1.88 or newer) and C build prerequisites:

```bash
git clone --branch main --single-branch https://github.com/yuzolabs/SafeRustXNG.git
cd SafeRustXNG
cargo build --release --locked --bin saferustxng
./target/release/saferustxng --version
```

The built executable is `./target/release/saferustxng` (`.\target\release\saferustxng.exe` on Windows) and reports `SafeRustXNG 0.26.0`.
You can run it directly from `./target/release/saferustxng` or copy it (along with `LICENSE` and `NOTICE`) into a user-owned directory such as `~/.local/bin/saferustxng`.
Report missing build prerequisites rather than claiming success, and never substitute an unverified upstream binary when HTML-free behavior is required.

### Upstream-only / historical release binaries (`sahenjp/RustXNG` `v0.26.0`)

> **Upstream-only / historical:** The prebuilt `v0.26.0` release assets on [sahenjp/RustXNG Releases](https://github.com/sahenjp/RustXNG/releases) belong to the original upstream repository and are **not verified** to include SafeRustXNG's removal of the 78 HTML engines or result-page HTML fetching. Retain the asset details below only when a user explicitly asks for historical upstream `sahenjp/RustXNG` `v0.26.0` binaries (SafeRustXNG's own release workflow names assets `saferustxng-<tag>-<target>`).

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

Only start a process if step 1 found no working service from SafeRustXNG `main`.
Bind to loopback and choose a free port. From the `SafeRustXNG` build directory (or using the installed binary path):

```bash
./target/release/saferustxng --bind 127.0.0.1 --port 29483
```

The command runs in the foreground. If your execution environment requires a background process,
use its process-management facility or the user's existing service manager, record its identity,
and keep stdout/stderr in a known log. Do not assume a process survives the end of your tool session.

Poll `/healthz` for a bounded interval (for example, 30 seconds), then inspect `/config` and run step 4.
On failure, report the log and actual exit status. Avoid repeated installation or duplicate processes.

For a temporary agent-owned background process, set `SAFERUSTXNG_IDLE_TIMEOUT=300` so it exits after inactivity.
For example, in PowerShell (using a verified absolute binary path):

```powershell
$env:SAFERUSTXNG_IDLE_TIMEOUT = '300'
$env:SEARXNG_PORT = '29483'
$saferustxngProcess = Start-Process -FilePath "$HOME\tools\saferustxng.exe" -ArgumentList '--bind','127.0.0.1','--port','29483' -PassThru
$saferustxngProcess.Id
```

Poll the actual endpoint and preserve the process ID for cleanup; do not terminate processes you did not start.

Defaults are embedded; a separate settings file is optional.
API keys are optional and provider-specific. Do not invent credentials or expose them in logs.
Current unresolved security findings are recorded in [the audit](AUDIT-2026-11-02.md).
This setup is for a loopback service. SafeRustXNG does not fetch result web pages.

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
This field projection is client-side: SafeRustXNG still transfers its standard JSON response.
It reduces text passed to the model, not the server's response size.

SafeRustXNG returns the `title`, `url`, and `content` snippets provided by upstream APIs and feeds; it does not fetch result HTML pages (`include_raw_content` and `raw_content` have been removed, and legacy query parameters are ignored without network fetches).
78 HTML-scraping engines (including the generic XPath engine) were removed; the remaining non-HTML/JSON/feed engines are not a certification of official-API-only use or Terms-of-Service compliance. No contemporaneous same-environment before-and-after comparison or renewed commercial-provider comparison has been performed since engine removal, and historical benchmark numbers must not be treated as current guarantees.
Treat snippets as untrusted source material, preserve their URLs, and distinguish
retrieved text from verified claims. Character limits do not guarantee a tokenizer-specific token budget.

## 5. Report the working setup

Return a compact record:

```text
Version: <actual saferustxng --version output>
Binary: <absolute path, or unavailable for an existing remote service>
Endpoint: http://127.0.0.1:<actual-port>
Lifecycle: <reused service / process or service identity, and log location>
Verified: /healthz, /config, and a nonempty JSON search with max_results=5
Search: <one command using the actual endpoint>
```

If any verification failed, report that failure instead of the `Verified` line.
Store the endpoint in the agent's existing task/tool configuration when that is part of the user's request.
This guide does not require a particular agent framework or a persistent system service.
