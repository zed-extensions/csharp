#!/usr/bin/env bash
# Manual version-bump convention for the C# Plus fork (D0.1).
#
# Upstream's generated `bump_version` workflow is gated on
# `repository_owner == 'zed-industries' || 'zed-extensions'` and invokes a
# reusable workflow that requires Zed-internal secrets. Neither works under
# the fork's owner, so the fork uses this script instead. This is the one
# documented version-bump mechanism; see docs/versioning.md.
#
# Usage:
#   scripts/bump-version.sh <major|minor|patch>   # bump from current version
#   scripts/bump-version.sh --set X.Y.Z           # set an explicit version
#
# The script bumps `version` in extension.toml and Cargo.toml, refreshes
# Cargo.lock, and prints the changelog entry you must add before committing.

set -euo pipefail

cd "$(dirname "$0")/.."

current() {
    sed -n 's/^version = "\(.*\)"$/\1/p' extension.toml | head -n1
}

bump() {
    local kind=$1 version
    version=$(current)
    IFS=. read -r major minor patch <<<"$version"
    case "$kind" in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) echo "unknown bump kind: $kind" >&2; exit 1 ;;
    esac
    echo "$major.$minor.$patch"
}

if [[ "${1:-}" == "--set" ]]; then
    new_version="${2:?usage: bump-version.sh --set X.Y.Z}"
elif [[ "${1:-}" =~ ^(major|minor|patch)$ ]]; then
    new_version=$(bump "$1")
else
    echo "usage: bump-version.sh <major|minor|patch|--set X.Y.Z>" >&2
    exit 1
fi

old_version=$(current)

sed -i '' "0,/^version = \"/s/^version = \"[^\"]*\"/version = \"$new_version\"/" extension.toml
sed -i '' "0,/^version = \"/s/^version = \"[^\"]*\"/version = \"$new_version\"/" Cargo.toml
cargo update --workspace --quiet

echo "bumped $old_version -> $new_version"
echo
echo "Next steps (do not skip):"
echo "  1. Add a CHANGELOG.md entry for $new_version, recording the"
echo "     upstream commit merged per the D0.1 sync policy (if any)."
echo "  2. Commit both version bumps, Cargo.lock, and the changelog together."
echo "  3. Open the publishing PR to zed-industries/extensions (docs/publishing.md)."
