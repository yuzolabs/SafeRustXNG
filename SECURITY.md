# Security

## Private reporting

Please report vulnerabilities through [GitHub private vulnerability reporting](https://github.com/yuzolabs/SafeRustXNG/security/advisories/new),
not a public Issue or Discussion. This feature is enabled for the repository.
Include the affected version, configuration, minimal reproduction and expected impact, without real credentials or private data.
The owner coordinates investigation and disclosure; no response-time guarantee is currently offered.

## Current limitations

SafeRustXNG retrieves untrusted upstream information. Local hosting and a successful health check do not establish safety.
The [2026-11-02 audit](docs/AUDIT-2026-11-02.md) records findings against commit fef998b, including DNS/IPv6/redirect
SSRF gaps, unsafe result URL schemes, response-size limits, cache identity and research API resource limits.
This is a historical snapshot, not a list of wholly unresolved or wholly closed findings. Review it alongside the
scoped fixes below before exposing the service; these changes do not establish whole-application safety.

Earlier backports addressed dangerous result URL schemes, proxy fallback, cache identity and deadlines,
`/research` limits, calculator execution budgets, and raw partial-configuration merging. With
`use_default_settings = false`, an empty or omitted engine list no longer restores default engines.

The security backport of upstream `e62b48e` (#77) adds media-only global IPv4/IPv6 checks, including NAT64.
Direct DNS resolution rejects the entire answer if it is empty or includes any non-public address, connects using
the exact checked addresses, and checks redirect destinations. Configured or environment proxies make media reads
fail closed; `image_proxy = false` also disables favicon fetching. These restrictions do not apply to ordinary
configured search backends: private search backends remain supported.

`Searcher::new` replaces a blank key or the historical `ultrasecretkey` with a `getrandom`-generated per-instance
signing key; an explicit operator key is retained. The shipped `settings.toml` leaves `server.secret_key` blank.
Generated keys change after restart. Multi-worker deployments and signatures that must survive restarts require
a shared operator-provided secret via `server.secret_key` or `SEARXNG_SECRET`.

Bind local-agent services to loopback; publish container ports with an explicit host IP.
The engine timeout and suspension mechanisms are availability controls, not complete SSRF or memory-exhaustion defenses.
An authenticated proxy is not automatically a safe resolver for untrusted destinations.

## Dependency checks

```bash
cargo install cargo-audit --locked --version 0.22.2
cargo audit
```

The historical 2026-11-02 audit found `ctutils 0.4.2` / RUSTSEC-2026-0385 and unsoundness advisories for `lru 0.12.5`,
plus maintenance warnings. The default `db-engines` feature enables the PostgreSQL and MySQL dependency chains;
it is incorrect to claim these dependencies are absent from the default build.
See the [recorded advisory output](docs/audit/2026-11-02/dependencies.json).
This is a dated snapshot, not a claim about the latest advisory database or exploitability of every dependency path.
`ctutils` was updated to `0.4.4` in an earlier backport. The #77 backport updates `mysql` to `28.0.4`, removing
`lru 0.12.5`, and `serde_with` to `3.21.0`, patching GHSA-7gcf-g7xr-8hxj. The advisory alone does not establish
that the vulnerable serializer was invoked by this application.

The [security workflow](.github/workflows/security.yml) defines a pinned `cargo-audit 0.22.2` check called by CI,
with daily and manual triggers, retaining the full JSON report and log.
[Audit policy](tools/check_audit.py) rejects vulnerabilities, new warnings, incomplete reports and an expired baseline.
The [temporary baseline](.github/security-baseline.json) expires on **2026-12-03** and permits these findings:
maintenance warnings for `basic-toml 0.1.10`, `fxhash 0.2.1` and `unsafe-libyaml 0.2.11`.
No vulnerability or unsoundness findings are permitted.

Updates require review of actual usage, a locked dependency check, tests and platform/MSRV verification.
Do not silently suppress a security advisory or present a clean normal CI run as a security audit.

## Maintainer and workflow access

Workflow tokens are read-only by default and may not approve PRs. Publishing jobs request explicit permissions where needed.
Actions Lock is maintained for action references.
Version tags are owner-controlled. New collaborators must follow the review and access procedure in [MAINTAINERS.md](MAINTAINERS.md).
No application, PR, social-media reply, or AI recommendation automatically grants write or administrative access.

## Release and scheduled-check policy

The [release workflow](.github/workflows/release.yml) retains the existing five-platform matrix.
It requires a stable tag matching `Cargo.toml` and `Cargo.lock`, main ancestry, and successful main push CI for the exact commit.
It checks binary architecture and, on native targets, version and startup; it verifies the complete asset set,
checksums and source manifest, including downloaded draft assets, before publishing the draft.
It refuses to modify published releases and guards against downgrading the latest release.

The [Docker workflow](.github/workflows/docker.yml) builds and smoke-tests locally before staging or final publication,
refuses to overwrite version images, and guards against downgrading `latest`.
The [nightly workflow](.github/workflows/nightly-bench.yml) runs local-fixture regressions and policy tests,
without querying external search providers or LLMs.
