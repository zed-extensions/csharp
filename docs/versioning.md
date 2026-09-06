# Versioning and releases

This fork replaces upstream's generated `bump_version` GitHub workflow with the
manual convention below.

## Why the workflow was removed

`.github/workflows/bump_version.yml` (generated from `xtask` inside the Zed
repository) was gated on
`github.repository_owner == 'zed-industries' || 'zed-extensions'` and invoked a
reusable workflow from `zed-industries/zed` requiring the Zed-internal
`ZED_ZIPPY_APP_*` secrets. Under the fork's owner it could never run, and even
ungated it would fail on the missing secrets. Decision (D0.1): **replace it
with a manual version-bump convention** so the repository has exactly one
workflow story. All CI now lives in `.github/workflows/ci.yml`.

## How to release

1. Merge `upstream/main` per the [sync policy](upstream-sync.md) and record the
   merged upstream commit in `CHANGELOG.md`.
2. Run `scripts/bump-version.sh <major|minor|patch| --set X.Y.Z>`. It bumps
   `version` in both `extension.toml` and `Cargo.toml` and refreshes
   `Cargo.lock`.
3. Add the `CHANGELOG.md` entry (commit it together with the bumps).
4. Open the publishing PR to `zed-industries/extensions` per
   [publishing.md](publishing.md).

## Release sequencing rules (from the backlog)

- M1.x ships as a single release; Razor semantic work (M3.4) must not ship in
  the same version as M1.5.
- Every user-visible release needs: a clean build, user-facing documentation,
  unchanged ordinary `.cs` behavior, and harness coverage proportionate to its
  scope.
