# Extension inventory and contract (M0.1)

Baseline of what the extension registers and how it behaves, and the mapping
from every later backlog item to the source and test files it touches. **This
table is the M0.1 deliverable**; update it as items land.

## Current contract (baseline at fork point `88597e1`)

### Language registrations

| Language | Path suffixes | Grammar | Tasks | Notes |
| --- | --- | --- | --- | --- |
| `CSharp` | `cs` | `tree-sitter-c-sharp` @ `485f0bae` | — | `autoclose_before = ";:.,=}])>"`, `//` and `///` comments, string-aware quote pairs |
| `C# Project File` | `csproj` | `tree-sitter-xml` @ `863dbc38` (xml) | restore/build current project | |
| `MSBuild File` | `proj`, `props`, `targets` | `tree-sitter-xml` (xml) | — | |
| `C# Solution File` | `slnx` | `tree-sitter-xml` (xml) | restore/build current solution | |

### Language servers (all attached to `CSharp` only, singular `language =` form)

| Server id | Install | Launch | Settings surface |
| --- | --- | --- | --- |
| `omnisharp` | GitHub release asset `OmniSharp/omnisharp-roslyn` (`omnisharp-{os}-{arch}-net6.0.{tar.gz,zip}`), cached dir `omnisharp-{version}` | binary directly, `-lsp` default arg; `which OmniSharp` short-circuit; `binary.path` override | `binary` only |
| `roslyn` | NuGet `roslyn-language-server.{win,linux,osx}-{x64,arm64}`, cached dir `roslyn-{version}`, self-contained exe or `dotnet exec` fallback (`any` rid) | `--stdio --autoLoadProjects` + user args; `binary.path` override | `binary`; `settings` mapped through `csharp\|` key flattening, inlay hints and reference/test code lenses defaulted |
| `csharp-ls` | NuGet `csharp-ls`, cached dir `csharp-ls-{version}`; `which csharp-ls` short-circuit; requires `dotnet` on PATH otherwise | `dotnet exec .../CSharpLanguageServer.dll` | `binary`; `settings` wrapped as `{"csharp": ...}` |

Supported platform matrix in code: `{win,linux,osx} × {x64,arm64}` (six RIDs),
plus a `dotnet exec` fallback path (`rid = any`). Extension API surface used:
`language_server_command`,
`language_server_workspace_configuration` (Roslyn, csharp-ls only),
`set_language_server_installation_status`, `download_file`,
`latest_github_release`, `make_file_executable`, `which`, `http_client`.

### What the extension does NOT do today

- No tests, no CI of its own (only the generated `bump_version.yml`).
- No Razor anything: no `.razor`/`.cshtml` registration, no grammar, no queries.
- No `.sln` (classic) association.
- No initialization-options hook usage.

## Backlog-item → file map

The authoritative mapping; every item below must keep this table accurate.

| Item | Source files touched | Test/verification files |
| --- | --- | --- |
| D0.1 | `.github/workflows/*`, `scripts/bump-version.sh`, `CHANGELOG.md`, `README.md`, `extension.toml` (authors), `docs/{versioning,upstream-sync,publishing}.md` | CI green; registry verification date in `docs/publishing.md` |
| M0.0 | — (spike; no product code) | `docs/feasibility-spike.md`, `scripts/spike/*.py` artifacts |
| M0.1 | `docs/inventory.md` (this file) | — |
| M0.2 | `extension.toml` (`[grammars.razor]`), `docs/razor-grammar-audit.md` | `harness/` fitness numbers, `corpus/razor/**` |
| M0.3a | `corpus/razor/**` | input to M0.2 + M1.0 harness |
| M0.3b | `fixtures/**`, `fixtures/README.md` | `scripts/verify-fixtures.sh`, release smoke |
| M0.4 | `docs/razor-contract.md`, `BACKLOG.md` (decision), `extension.toml` (per decision) | M1.5 smoke: no server for Razor |
| M1.0 | `harness/**`, `.github/workflows/ci.yml`, `scripts/snapshot.sh` | golden outputs `harness/golden/**`; deliberate-regression check |
| M1.1 | `extension.toml`, `languages/razor/config.toml` | M1.0 harness; M1.5 smoke |
| M1.2 | `languages/razor/{highlights,injections}.scm` | `harness/golden/razor/**` |
| M1.3 | `languages/razor/{brackets,indents,outline,textobjects,runes}.scm`, `languages/razor/config.toml` | `harness/golden/razor/**` (outline/textobjects/indents snapshots) |
| M1.4 | `languages/razor/snippets.json`, `README.md`, `docs/{migration,known-limits}.md` | manual smoke checklist |
| M1.5 | `CHANGELOG.md`, `extension.toml` + `Cargo.toml` versions, `docs/publishing.md`, `docs/release-smoke.md` | macOS/Linux/Windows smoke, dated in changelog |
| M2.1 | `src/language_servers/roslyn.rs` (error messages), `docs/dotnet-workflow.md` | manual: each failure keeps files editable; documented remediation |
| M2.2 | `languages/csproj/tasks.json`, `languages/slnx/tasks.json`, `docs/dotnet-workflow.md` | task definitions against `fixtures/**` (release smoke) |
| M2.3 | `docs/debugging.md` | dated doc decision |
| M2.4 | `docs/project-files.md`, possibly `languages/msbuild/config.toml` | documented associations |
| M3.1 | `src/language_servers/roslyn.rs` + `razor_support.rs`, `docs/roslyn-razor-pinning.md` | reproducible startup script `scripts/spike/*`; version-mismatch error |
| M3.2 | `scripts/probe_razor.py`, `src/language_servers/razor_support.rs` | `docs/razor-probe-results.md` evidence table |
| M3.3 | `docs/upstream-proposal.md` (discussion artifact) | outcome recorded in `BACKLOG.md` G2 |
| M3.4 | `extension.toml` (`language_ids` map), `src/language_servers/roslyn.rs` (initialization options gating), `CHANGELOG.md` (own release) | opt-in/opt-out documented and demonstrated |
| M4.1 | `docs/gate-g2-decision.md` | decision record completeness |
| M4.2A | (conditional; not taken) | — |
| M4.2B | `proxy/**` (separate crate), `src/language_servers/proxy.rs` | `proxy/tests/**` range-mapping + conformance fixtures; kill-test |
| M4.3 | `docs/quality-gates.md`, `CHANGELOG.md` (own release), `src/language_servers/*` default flip | gate matrix executed |
| Quality gates | `docs/{quality-gates,manual-checklist,release-smoke}.md`, `harness/`, `.github/workflows/ci.yml`, `scripts/release-smoke.sh` | the gates themselves |
