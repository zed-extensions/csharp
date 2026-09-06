# Changelog

All notable changes to the C# Plus fork are documented here. Each entry that
followed an upstream sync records the merged upstream commit (D0.1 policy).

## Unreleased

### 1.3.0 — Razor editing, workflow, and foundation (Milestone 0 + 1 + 2)

The first fork release. Delivers the Milestone 0 foundation (feasibility,
corpus, fixtures, contract), first-class Razor *editing* (Milestone 1), and the
everyday .NET workflow improvements (Milestone 2). This is editing support, not
full Razor IDE semantics: **no language server starts for Razor buffers in
this release** (see `docs/razor-contract.md`).

#### Added
- Razor language (`Razor`) registered for `.razor` and `.cshtml` with the
  pinned `tree-sitter-razor` grammar (M1.1; grammar decision in
  `docs/razor-grammar-audit.md`).
- Razor syntax highlighting, HTML/CSS/JS injections, bracket matching,
  auto-indentation, outline, and text objects (M1.2, M1.3).
- Razor snippets for common directives and blocks (M1.4).
- Highlight snapshot harness over the committed Razor corpus plus the existing
  `csharp` and `msbuild` queries, run in CI on every push (M1.0).
- Discoverable `dotnet` task templates: restore, build, test, run, watch,
  clean, publish, format, and EF Core migrations (M2.2).
- Roslyn-first project guidance and actionable diagnostics for the common
  failure modes (M2.1).
- Documentation: migration off the upstream C# extension, debugging state,
  supported files and known limits (M1.4, M2.3, M2.4).

#### Fork infrastructure (D0.1)
- Replaced upstream's `bump_version` workflow (gated on Zed org ownership and
  Zed-internal secrets; could never run in the fork) with the manual
  convention in `scripts/bump-version.sh` + `docs/versioning.md`.
- Added the highlight/CI workflow story in `.github/workflows/ci.yml`.
- Recorded upstream sync policy and registry verification in `docs/`.

#### Decisions recorded
- G0 (M0.0 outcome), G1 (M0.2 grammar decision), and the M0.4 opt-in
  mechanism are recorded in `BACKLOG.md` and the referenced docs.

## 1.2.2

Fork point: upstream `zed-extensions/csharp` at commit `88597e1` (v1.2.2).
Upstream history before this entry belongs to the upstream project.
