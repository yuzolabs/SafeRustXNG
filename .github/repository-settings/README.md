# Repository settings record

These JSON files hold repository settings payloads for `yuzolabs/SafeRustXNG`. Committing them does **not** automatically apply settings.
Do not run an unreviewed settings file from a contributor branch with an administrator token.

- `repository.json`: desired repository-level settings payload (Discussions enabled, squash-only merges, automatic deletion of merged branches, auto-merge availability).
- `main-protection.json`: applied `main` branch protection (`strict=true` required checks from GitHub Actions app `15368`, PR workflow,
  conversation resolution, linear history, no force pushes or branch deletion).
- `release-tags.json`: desired (currently unapplied) tag ruleset restricting `refs/tags/v*` creation, update, and deletion, with only the owner account (`User` `240885066`, `yuzolabs`, `bypass_mode: "always"`) as bypass actor.

The `main` approval count is initially zero (`required_approving_review_count=0`) because the owner is the only maintainer. Administrator emergency bypass
is retained (`enforce_admins=false`); this is not an unrestricted bypass for ordinary write collaborators.
Before the first write invitation, follow [MAINTAINERS.md](../../MAINTAINERS.md) and require one approval.

Required status check contexts in `main-protection.json` match the jobs in [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml): `test (ubuntu-latest)`, `test (macos-latest)`, `test (windows-latest)`, `no-default-features`, `msrv`, and `docker` (all with GitHub Actions `app_id: 15368`).

Other repository-level policies (such as Actions workflow token permissions, private vulnerability reporting, and fork workflow approval policy) are not managed by the JSON files in this directory; verify them directly in the live repository settings rather than relying on the original upstream `sahenjp/RustXNG` setup record.

An owner can inspect and reconcile repository and `main` branch protection settings after reviewing the live values:

```bash
gh api repos/yuzolabs/SafeRustXNG
gh api repos/yuzolabs/SafeRustXNG/branches/main/protection
# Writes, only after reviewing the JSON:
gh api --method PATCH repos/yuzolabs/SafeRustXNG --input .github/repository-settings/repository.json
gh api --method PUT repos/yuzolabs/SafeRustXNG/branches/main/protection --input .github/repository-settings/main-protection.json
```

Ruleset ID `26350142` belonged to the upstream repository (`sahenjp/RustXNG`), not `yuzolabs/SafeRustXNG` (where `gh api repos/yuzolabs/SafeRustXNG/rulesets` currently returns `[]`). List existing rulesets before creating one so you do not create duplicate tag rulesets. When no release-tag ruleset exists yet, create it with `POST`, capture and store the `id` returned by GitHub, and use that verified ID for any later `GET` or `PUT`:

```bash
# 1. List existing rulesets first; do not create a duplicate if one already exists:
gh api repos/yuzolabs/SafeRustXNG/rulesets

# 2. Only when no release-tag ruleset exists yet, create it and record the returned "id":
gh api --method POST repos/yuzolabs/SafeRustXNG/rulesets --input .github/repository-settings/release-tags.json

# 3. For later inspection or updates, set RULESET_ID to the verified ruleset ID returned by GitHub:
# RULESET_ID=<verified-ruleset-id>
gh api "repos/yuzolabs/SafeRustXNG/rulesets/${RULESET_ID:?Set RULESET_ID to the verified ruleset ID}"
gh api --method PUT "repos/yuzolabs/SafeRustXNG/rulesets/${RULESET_ID:?Set RULESET_ID to the verified ruleset ID}" --input .github/repository-settings/release-tags.json
```

If ownership or the active maintainer roster changes, review the bypass account, CODEOWNERS, and approval count together.
