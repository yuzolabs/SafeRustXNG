# Roadmap and contribution areas

Updated 2026-11-02. This records priorities and review gates, not promised delivery dates.
New features remain proposals until code review, current-head CI and the stated evidence are complete.

## First: make the foundation reliable

| Area | Priority / gate |
| --- | --- |
| Published [audit](AUDIT-2026-11-02.md) | Resolve untrusted fetching, URL schemes, response size limits and resource controls before expanding network features |
| Request/cache identity and deadlines | Isolate cache keys across authentication and request parameters, and enforce end-to-end deadlines before adding more authenticated or cached providers |
| Input validation and engine state visibility | Validate numeric query parameters and expose resting/suspended engine state clearly in responses |

## Community feedback translated into work

| Work | Completion criteria |
| --- | --- |
| [#39 Linux ARM64 distribution](https://github.com/sahenjp/RustXNG/issues/39) | Verified native assets, checksums, startup/search smoke tests and accurate install instructions; distinguish ARM64 from ARMv7 |
| [#40 Japanese errors and broad/fresh retrieval benchmark](https://github.com/sahenjp/RustXNG/issues/40) | Public fixed queries, reference URLs, repetitions, failures, language/time/cache conditions; separate pinpoint and broad-source metrics |
| [#41 agent output budgets](https://github.com/sahenjp/RustXNG/issues/41) | Compact fields, source retention, explicit truncation and well-defined character/byte/token budgets |
| [#30 built-in MCP](https://github.com/sahenjp/RustXNG/issues/30) | Stdio transport, bounded search output, current SDK/MSRV compatibility and outbound request/proxy policy |

Rust implementation, retrieval quality, transfer cost and model context consumption are different dimensions.
See [FAQ](FAQ.md). Keep the existing [SealQA retrieval baseline](SEALQA.md) reproducible;
new workloads must not inherit its scores as a performance guarantee.

## Review gates for future changes

| Area | Gate |
| --- | --- |
| Engine applicability and query routing | Verify engine capability checks and offline fixture tests (`cargo check --tests`) before enabling routing changes |
| Language and locale parameters | Define and test `auto` and `all` consistently across both `language` and locale code paths, including Japanese queries |
| Reference infobox presentation | Preserve the agent `results` path and explicit Wikipedia search, or provide an opt-in display mode |
| Additional API providers | Require authentication-aware cache isolation, quota/backoff handling, default-category review and terms-of-service compatibility before adding new API engines |

A blank CI list is not a successful run. Review new contributor workflows before approving their execution;
do not use privileged `pull_request_target` execution of untrusted checkout code as a shortcut.

## Areas seeking participation

Documentation and translations, offline engine fixtures, benchmark reproduction, native ARM64 distribution,
CI, observability and network-fetching safety are useful contribution areas.
Use [CONTRIBUTING.md](../CONTRIBUTING.md), [Discussions](https://github.com/yuzolabs/SafeRustXNG/discussions),
or the [maintainer application](https://github.com/yuzolabs/SafeRustXNG/issues/new?template=maintainer_application.yml).
Participation is open; privileges require an explicit owner decision under [GOVERNANCE.md](../GOVERNANCE.md).
