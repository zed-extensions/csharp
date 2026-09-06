# Upstream sync policy (D0.1)

`upstream` points at `zed-extensions/csharp`; `origin` points at the fork
(`joeizang/zed-csharp-plus`).

## Policy

- Fetch and merge `upstream/main` **at the start of every milestone** and
  **before any release**.
- Record the merged upstream commit in `CHANGELOG.md` for that release.
- Divergence is cheapest to resolve while small. The files most likely to
  conflict are:
  - `languages/csharp/*.scm`
  - `src/language_servers/roslyn.rs`
  - `extension.toml`

```sh
git fetch upstream && git merge upstream/main
```

## Attribution obligations (must survive every merge)

- `LICENSE` (Apache-2.0) is retained unmodified. The licence requires that
  notices be preserved and modifications be marked.
- Upstream authors stay listed in `extension.toml`.
- Fork provenance and the fork-point commit stay in `README.md`.
- Copied/adapted assets (grammar queries, fixtures) carry their own
  attribution headers — see [razor-grammar-audit.md](razor-grammar-audit.md).

If a merge would drop any of the above, the merge is wrong; fix it before
committing.

## Registry

Publishing requires a PR to `zed-industries/extensions` adding `csharp-plus`
as a new entry. The id was verified free on 2026-09-05 (the registry then
contained `csharp` and `csharp-snippets`, but no `csharp-plus`); re-verify
before M1.5 per docs/publishing.md.
