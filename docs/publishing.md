# Publishing

## One-time registry entry

Publishing requires a PR to [zed-industries/extensions](https://github.com/zed-industries/extensions)
adding `csharp-plus` as a new entry in `extensions.toml`:

```toml
[csharp-plus]
submodule = "extensions/csharp-plus"
version = "1.3.0"
```

The id `csharp-plus` was verified free on **2026-09-05** (the registry then
contained `csharp` and `csharp-snippets` only). Re-verify before opening the
PR; if it has been taken, stop and reconsider the extension id before
releasing — a renamed id after first publish breaks upgrades.

## Per-release checklist

1. Upstream merged per [upstream-sync.md](upstream-sync.md); commit recorded in
   the changelog.
2. Version bumped via `scripts/bump-version.sh` (see [versioning.md](versioning.md)).
3. CI green on the release commit (harness + builds on all three OSes).
4. Release smoke pass per [release-smoke.md](release-smoke.md) on the
   developer's primary OS; the other two OSes covered by CI-executable smoke.
5. Manual checklist from [manual-checklist.md](manual-checklist.md) executed
   and dated in the changelog entry.
6. Publishing PR opened; merge order: registry PR after the version commit
   lands on `origin/main`.
