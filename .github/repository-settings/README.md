# Repository settings record

These JSON files hold repository settings payloads for `yuzolabs/SafeRustXNG`. Committing them does **not** automatically apply settings.
Do not run an unreviewed settings file from a contributor branch with an administrator token.

- `repository.json`: desired repository-level settings payload (Discussions enabled, squash-only merges, automatic deletion of merged branches, auto-merge availability).
- `main-protection.json`: applied `main` branch protection (`strict=true` required checks from GitHub Actions app `15368`, PR workflow,
  conversation resolution, linear history, no force pushes or branch deletion).
- `release-tags.json`: applied tag ruleset (ID `26374379`, `Release tags: owner-controlled`, `enforcement: "active"`) restricting `refs/tags/v*` creation, update, and deletion, with only the owner account (`User` `240885066`, `yuzolabs`, `bypass_mode: "always"`) as bypass actor.

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

The active release-tag ruleset on `yuzolabs/SafeRustXNG` has ID `26374379` (upstream `sahenjp/RustXNG` used ruleset ID `26350142`). Because the owner account (`User` `240885066`, `yuzolabs`) has `bypass_mode: "always"`, non-bypass actors cannot create, update, or delete `refs/tags/v*`, but the owner can still move or delete `v*` tags and this ruleset does not make GitHub Release assets immutable. List and inspect existing ruleset `26374379` before updating it in place with `PUT` (do not run `POST` when the ruleset already exists, as that creates a duplicate; use `POST repos/yuzolabs/SafeRustXNG/rulesets` only to recreate the ruleset if `gh api repos/yuzolabs/SafeRustXNG/rulesets` shows it is absent, and record the new ID):

```bash
# List rulesets and inspect the applied release-tag ruleset (ID 26374379):
gh api repos/yuzolabs/SafeRustXNG/rulesets
gh api repos/yuzolabs/SafeRustXNG/rulesets/26374379

# Update the existing ruleset in place, only after reviewing the JSON (do not POST a duplicate):
gh api --method PUT repos/yuzolabs/SafeRustXNG/rulesets/26374379 --input .github/repository-settings/release-tags.json
```

If ownership or the active maintainer roster changes, review the bypass account, CODEOWNERS, and approval count together.
