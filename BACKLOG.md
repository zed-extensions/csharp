# C# Web Development Backlog

## Product direction

Make this the integrated C# web-development extension for Zed. It owns C#,
Razor, .NET project files, and the everyday .NET workflow; existing Zed
extensions continue to own HTML, CSS, JavaScript, TypeScript, and debugging.

## Decision 0 — Delivery target (DECIDED: fork)

This work ships as a **fork** of `zed-extensions/csharp`, published as a
distinct extension: id `csharp-plus`, name "C# Plus", crate `zed_csharp_plus`,
forked at upstream commit `88597e1` (v1.2.2). The alternatives — upstreaming
into `zed-extensions/csharp`, or shipping a Razor-only extension alongside it —
were considered and rejected.

**Why the fork wins.** The product goal is one integrated extension owning C#,
Razor, project files, and the .NET workflow. A Razor-only extension cannot
deliver that: from Milestone 3 onward it would have to launch its own
`Microsoft.CodeAnalysis.LanguageServer`, giving two Roslyn processes over one
solution — double project load, double memory, two divergent views of the same
compilation. Upstreaming avoids that but puts every step behind review latency
this project does not control, on a roadmap whose feasibility (G0) is not yet
established; a maintainer who declines Razor scope at M1, or a proxy binary at
M4, invalidates the plan retroactively. The fork keeps the integrated product
*and* full control of sequencing, and it inherits working OmniSharp, csharp-ls,
NuGet-download and MSBuild code rather than reimplementing it.

**The cost, stated plainly.** C# Plus is a superset of the upstream extension,
not a companion to it. It registers `.cs`, `.csproj`, `.slnx` and MSBuild files,
so installing both produces duplicate language ownership. Users must uninstall
"C#" and install "C# Plus" — a one-time, documented migration (see M1.4), not a
silent conflict, which keeps faith with the decision below that this project
does not modify a user's installed extensions. The fork also inherits
maintenance of the OmniSharp, `csharp-ls` and NuGet code paths, and takes on a
merge burden against upstream. Both are managed by D0.1 rather than wished away.

- [ ] **D0.1 — Establish fork hygiene before M1 starts**
  - `upstream` remote points at `zed-extensions/csharp`; `origin` points at the
    fork. (Remote rename done; `origin` is added once the GitHub fork exists.)
  - **Upstream sync policy:** fetch and merge `upstream/main` at the start of
    every milestone, and before any release. Record the merged upstream commit
    in the changelog. Divergence is cheapest to resolve while small; the
    files most likely to conflict are `languages/csharp/*.scm`,
    `src/language_servers/roslyn.rs`, and `extension.toml`.
  - **Attribution:** LICENSE retained unmodified; upstream authors retained in
    `extension.toml`; provenance and fork commit recorded in README. Apache-2.0
    requires the notices be preserved and modifications be marked — done, and
    it must stay done through every merge.
  - **CI is currently dead in the fork.** `.github/workflows/bump_version.yml`
    is gated on `repository_owner == 'zed-industries' || 'zed-extensions'`, so
    it will never run under the fork's owner. Either adopt the gate to the fork
    owner or replace it with a manual version-bump convention. Decide before
    M1.0 adds the first real CI job, so there is one workflow story rather than
    two.
  - **Registry:** publishing requires a PR to `zed-industries/extensions`
    adding `csharp-plus` as a new entry. Verify the id is free before M1.5.
  - Acceptance: a clean `git merge upstream/main` runs green, the licence and
    attribution obligations are satisfied, and the version-bump story is one
    documented mechanism.

## Decisions already made

- Support both ASP.NET Core MVC/Razor Pages (`.cshtml`) and Blazor (`.razor`).
- Deliver in small releases: Razor editing, workflow improvements, experimental
  Razor semantics, then stable semantics.
- Use Roslyn as the only supported semantic backend for Razor. OmniSharp and
  `csharp-ls` remain C#-only alternatives.
- Guarantee .NET 8 LTS and newer. Legacy ASP.NET/.NET Framework projects may
  receive syntax support but are not promised project-aware Razor semantics.
- Syntax editing is on by default. Razor semantics remain opt-in until stable.
  See M0.4: the *mechanism* for this is not yet known to exist.
- On unsupported SDK/project/server conditions, preserve syntax editing and
  explain the next action; never silently switch language servers.
- Reuse the existing Razor extension's assets only after recording the source,
  commit, licence, notices, and any local modifications.
- Do not modify a user's settings or installed extensions. Explain Razor file
  association conflicts and provide migration instructions instead.
- Do not add a debugger, application scaffolding, deployment/cloud tooling, or
  legacy-framework semantic support in this roadmap.
- Do not collect telemetry or upload project contents. Opening a Razor file
  must not itself trigger restore or build work.

### Platform constraints taken as given

These are properties of Zed's extension model, not open questions. They are
recorded here because two milestones below were originally written as though
they did not hold.

- A Zed extension is a WASM plugin. Its entire LSP surface is
  `language_server_command`, `language_server_initialization_options`,
  `language_server_workspace_configuration`, the two `additional_*` variants,
  and `label_for_completion` / `label_for_symbol`. There is no hook to observe,
  rewrite, or synthesise LSP traffic, and no concept of a virtual or projected
  document. See zed-industries/zed#21133, closed as not planned.
- Consequently, any Razor request/range bridging must run in a **separate proxy
  executable** launched by `language_server_command`, which itself spawns
  Roslyn and speaks LSP on both sides. See M4.2B.
- Zed only ever opens real files, so such a proxy owns the virtual-document
  lifecycle entirely — no virtual document can leak into ordinary editor
  workflows unless the proxy emits one.
- Attaching a server to a language is a manifest fact (`languages = [...]` plus
  a `language_ids` map). Opting out is a user-side setting
  (`"language_servers": ["!roslyn"]`), not an extension-side toggle.
- Extensions cannot override query files shipped in a grammar's own repository
  (zed#40532, closed as not planned). Queries under `languages/razor/*.scm` in
  this repository do load; upstreaming query fixes to a grammar repo is not a
  path.

## Definition of done

Every release must build cleanly, include user-facing documentation, preserve
ordinary `.cs` behavior, and have regression coverage proportionate to its
scope — where "coverage" means the harness built in M1.0, not an aspiration.
Razor semantic support cannot be called stable until it passes the matrix in
[Quality gates](#quality-gates).

Each user-visible release also requires a version bump in `extension.toml` and
`Cargo.toml` (the `bump_version` workflow handles this on merge) and a
publishing PR to `zed-industries/extensions`. Batch M1.x into a single release;
do not ship M3.4 in the same version as M1.5.

## Decision gates

The plan has three points where a negative answer changes the roadmap rather
than delaying it. Each has a written else-branch. Do not proceed past a gate
without recording the outcome in this file.

| Gate | After | Question | Else-branch |
| --- | --- | --- | --- |
| G0 | M0.0 | Can the shipped `roslyn-language-server` package serve Razor at all? | See M0.0's three outcomes |
| G1 | M0.2 | Which band does the best candidate land in? | Adopt / fork and vendor / descope M1 to injection-only |
| G2 | M4.1 | Upstream route accepted within 30 days of a complete M3.3? | Extension-owned proxy route, or stop at experimental |

M3.3 additionally times out on its own: if there is no substantive Zed staff
response within 30 days of posting, treat it as a decline for planning purposes
and start the G2 clock.

## Milestone 0 — Feasibility, foundation, and compatibility audit

Goal: answer the questions that decide whether Milestones 3 and 4 exist, then
remove the remaining unknowns before changing user-visible behavior.

- [ ] **M0.0 — Razor/Roslyn feasibility spike (1–2 days, do this first)**
  - Take the `roslyn-language-server.<rid>` package this extension already
    downloads. Launch it by hand with `--stdio --autoLoadProjects` plus
    `--razorSourceGenerator` and `--razorDesignTimePath`, against one Blazor
    page and one `.cshtml` page.
  - Drive it with a raw LSP client (a script is fine): `initialize`,
    `textDocument/didOpen` with language id `aspnetcorerazor`, then
    `textDocument/completion` and `textDocument/hover`.
  - Context: dotnet/roslyn#82535 establishes that these flags exist but are
    undocumented, and there is no maintainer statement that the standalone
    `Microsoft.CodeAnalysis.LanguageServer` package serves Razor outside the
    VS Code C# extension, which ships the Razor components separately.
  - Acceptance: a written outcome, one of —
    - **Works.** M3 is real; proceed as planned and record the exact flag
      values and component versions in M3.1.
    - **Needs components not in the package.** M3.1 becomes "acquire, licence,
      and repackage the Razor components", a materially larger and
      licence-sensitive job. Re-cost M3 before committing.
    - **Does not work.** M3 and M4 collapse to "track upstream". M1 and M2
      become the whole roadmap — an acceptable product outcome. Record it and
      move on rather than retrying against newer packages ad hoc.

- [ ] **M0.1 — Inventory the extension's current contract**
  - Document language registrations, server installation/launch behavior,
    project tasks, supported platforms, and user settings.
  - Capture a baseline for `.cs`, `.csproj`, MSBuild, and `.slnx` behavior.
  - Acceptance: a committed table mapping every later backlog item to the
    source files and test files it touches. The table is the deliverable; a
    prose audit is not.

- [ ] **M0.2 — Audit Razor grammar fitness and licence compatibility**
  - There is no canonical `tree-sitter-razor`. Candidates are small,
    lightly-maintained third-party repositories (noundry/zed-razor,
    IbrahimSabriOrene/zed-razor-treesitter, NoisKung/razor-syntex-zed).
    Pinning one means adopting its bugs permanently.
  - Verify licences for the grammar and every copied/adapted query file; add
    required attribution/notices and record the upstream source revision.
  - Decide now, in writing, whether this project is willing to maintain a Razor
    grammar fork indefinitely. On current evidence it probably will have to.
  - Acceptance: licence obligations are explicit, the grammar revision is
    reproducible by automation, and the fitness numbers plus the fork decision
    are recorded here.

  **Fitness test and thresholds (set before running, not after).**

  Parse each candidate over the M0.3a corpus at a pinned revision and record,
  per file: whether the tree contains any `ERROR` or `MISSING` node, and the
  share of file bytes covered by `ERROR` nodes. The two halves of the corpus
  are judged differently, because a deliberately broken file *should* produce
  errors.

  *Core construct subset* — roughly fifteen well-formed files that must parse
  cleanly for the extension to be worth shipping at all: implicit expression
  (`@model.Name`), explicit expression (`@(a + b)`), `@code` and `@functions`
  blocks, `@if` / `@foreach` / `@switch`, the directive set (`@page`, `@model`,
  `@using`, `@inject`, `@inherits`, `@attribute`), a component with parameters
  (`<Foo Bar="@x" />`), a Tag Helper (`asp-for`), `@bind` and `@onclick`,
  layouts and `@section`, Razor comments (`@*…*@`), the `@@` escape, an email
  address in body text, an `@media` rule inside `<style>`, and an `@` inside a
  `<script>` block.

  On the **well-formed** half, three measures are applied together:

  | Band | Criteria | Action |
  | --- | --- | --- |
  | Adopt as-is | ≤5% of files contain any `ERROR`/`MISSING`, worst-file error span ≤10% of bytes, and **zero** core-construct failures | Pin upstream, no fork |
  | Fork and fix | >5% and ≤25% of files affected, and ≤3 core-construct failures | Fork and vendor; the gaps are specific, nameable constructs |
  | Reject | >25% of files affected, or >3 core-construct failures, or any single `ERROR` node spanning >50% of a file | Try the next candidate |

  The >50%-span rule is separate on purpose: it means the parser derails and
  never recovers, which no amount of query work repairs.

  On the **pathological** half, errors are the expected outcome, so the gate is
  *containment* rather than absence: the `ERROR` node must not extend to
  end-of-file in more than 20% of those files, and constructs following the
  broken one must still parse to their correct node types. This is M1.2's
  "recover gracefully from incomplete templates" made measurable.

  If every candidate lands in Reject, G1's else-branch applies: descope M1 to a
  hand-written injection-only approach, or write a grammar.

- [ ] **M0.3a — Razor file corpus (gates M1)**
  - A directory of `.razor` and `.cshtml` files only — no projects, no restore.
  - Cover directives, Tag Helpers, layouts/partials, components and parameters,
    scoped CSS, JS interop, and deliberately pathological cases: unterminated
    blocks, mismatched tags, `@` in text and CSS, half-typed component tags.
  - Acceptance: the corpus is the input to both M0.2's fitness test and M1.0's
    snapshot harness, and contains no secrets.

- [ ] **M0.3b — Buildable fixture solutions (gates M2 and M3)**
  - Minimal restorable fixtures for MVC, Razor Pages, Blazor Web App, Blazor
    WASM, Razor Class Library, and a multi-project solution, plus generated and
    project-reload cases.
  - Acceptance: fixtures open and restore deterministically on .NET 8+, offline
    after a first restore, with no external-service dependency.

- [ ] **M0.4 — Specify the Razor compatibility contract**
  - Define `Razor` as a separate Zed language associated with `razor` and
    `cshtml` suffixes.
  - **Resolve the opt-in contradiction.** Attaching Roslyn to Razor requires
    `extension.toml` to carry `languages = ["CSharp", "Razor"]` under
    `[language_servers.roslyn]` plus a `[language_servers.roslyn.language_ids]`
    map (`"Razor" = "aspnetcorerazor"`). The moment `"Razor"` appears there,
    Zed starts Roslyn for every Razor buffer — so M1.5 ("editing only") and
    M3.4 ("add an opt-in setting") cannot both hold under one manifest.
    Determine whether extension-side conditional attachment is possible at all
    (e.g. returning an error from `language_server_command` when a worktree
    setting is absent — note this surfaces as an error, not a graceful no-op).
    If it is not, then either M1 registers Razor with **no** server attached and
    "opt-in" means "a later release", or "opt-in" means "a documented settings
    edit". Pick one and write it down; an unwary M1 otherwise ships
    experimental semantics silently.
  - Define user-visible states: editing-only, experimental semantics, stable
    semantics, unsupported environment, and conflicting extension.
  - Acceptance: each state has an exact message, enabled capabilities, and a
    recovery action; the opt-in mechanism is named and demonstrated.

## Milestone 1 — First-class Razor editing

Goal: make Razor files pleasant and structurally correct to edit without
depending on a Razor language server. Gated on M0.3a and G1.

- [ ] **M1.0 — Build the highlight snapshot harness**
  - This repository currently has no tests: `.github/workflows/` contains only
    the generated `bump_version.yml`, and `src/` has no test module. Every
    "Acceptance: fixture snapshots…" below depends on this item existing first.
  - Script the `tree-sitter` CLI (`highlight` / `query`) against the pinned
    grammar plus this repository's `.scm` files over the M0.3a corpus, with
    committed golden output and a diff-on-failure mode.
  - Run it in CI on push. Extend it to the existing `csharp` and `msbuild`
    queries so the C# regression blocker below is actually enforced.
  - Acceptance: a deliberate query regression fails CI. If this item is
    descoped, downgrade every snapshot acceptance criterion in M1 to an honest
    manual checklist rather than leaving it unverifiable.

- [ ] **M1.1 — Register the Razor grammar and language**
  - Add a pinned Razor grammar registration to `extension.toml`.
  - Migrate all three `[language_servers.*]` entries from the singular
    `language = "CSharp"` form to the plural `languages = [...]` form, which is
    the prerequisite for any `language_ids` mapping. All three are inherited
    from upstream, so this is a merge-conflict hotspot — see D0.1.
  - Add `languages/razor/config.toml` with `.razor` and `.cshtml` associations,
    comments, bracket behavior, and safe editing defaults.
  - Attach Razor to a language server only as M0.4 decided.
  - Acceptance: opening either suffix selects `Razor`; ordinary `.cs` remains
    `CSharp`; no language server starts for Razor buffers unless M0.4 says it
    should.

- [ ] **M1.2 — Implement syntax highlighting and injections**
  - Add Razor query files for directives, expressions, control flow, comments,
    C# blocks, and component/tag syntax.
  - Inject HTML and coordinate embedded C# with existing language definitions.
  - Preserve normal HTML/CSS/JS/TypeScript extension ownership.
  - Note: graceful recovery from incomplete templates is a property of the
    grammar chosen in M0.2, not of these queries. Failures here route back to
    the fork decision.
  - Acceptance: M1.0 snapshots correctly style Razor, C#, and HTML across the
    corpus, including the pathological files.

- [ ] **M1.3 — Implement structural editing support**
  - Add bracket matching, auto-indentation, syntax overrides, outline, and
    text objects where grammar support permits.
  - Verify `@code`, `@section`, `@if`, `@foreach`, components, HTML elements,
    Razor comments, and quoted attribute values.
  - Acceptance: no auto-close/indent behavior corrupts a mixed-language file.

- [ ] **M1.4 — Add focused Razor snippets and documentation**
  - Provide conservative snippets for common directives and blocks; do not
    replace framework scaffolding.
  - Document supported files, known limits, settings, and coexistence/migration
    from the separate Razor extension, and — the important one — the migration
    off the upstream "C#" extension: uninstall it, install C# Plus, existing
    `lsp.*` settings carry over unchanged. This is the doc that prevents the
    duplicate-ownership problem Decision 0 accepted.
  - Acceptance: a new user can install the extension, open a Razor project,
    and understand the available editor features without trial-and-error.

- [ ] **M1.5 — Release Razor editing support**
  - Changelog: describe this as first-class *editing* support, not full Razor
    IDE semantics.
  - Version bump plus publishing PR to `zed-industries/extensions` adding the
    `csharp-plus` entry, and the upstream merge required by D0.1.
  - Acceptance: manual smoke tests pass on macOS, Linux, and Windows; no
    duplicate file association is silently introduced; no language server
    starts for Razor buffers.

## Milestone 2 — Everyday .NET workflow

Goal: make standard C# web work natural from a Zed project window. Gated on
M0.3b.

- [ ] **M2.1 — Roslyn-first project guidance**
  - Document and test Roslyn as the recommended default for C# and required
    backend for Razor semantics.
  - Improve actionable diagnostics for missing SDKs, invalid `global.json`,
    project-load failure, incompatible Roslyn components, and disabled Roslyn.
  - Acceptance: each failure keeps files editable and identifies a concrete
    remediation path.

- [ ] **M2.2 — Add safe, parameterized .NET tasks**
  - Provide discoverable restore, build, test, run, watch, and migration task
    templates where Zed's task model supports them, extending the existing
    `languages/csproj/tasks.json` and `languages/slnx/tasks.json`.
  - Avoid running commands automatically and never expose connection strings or
    launch-profile secrets.
  - Acceptance: task definitions work against the M0.3b fixtures and are
    documented for multi-project selection.

- [ ] **M2.3 — Debugger handoff (conditional on a third-party adapter)**
  - This item's original acceptance criterion depended on a .NET debug adapter
    for Zed existing and working — which this project neither owns nor can
    deliver. It is therefore conditional.
  - If a usable adapter exists: document companion setup, task conventions, and
    expected `launchSettings.json` behavior, and verify the documented path
    launches and debugs a fixture app without debugger code being added here.
  - If none exists: document the current state plainly, including that
    debugging may not work, and how to troubleshoot a missing or incompatible
    adapter. Do not block Milestone 2 on it.
  - Acceptance: whichever branch applies is documented and dated.

- [ ] **M2.4 — Project-file ergonomics audit**
  - Confirm `.csproj`, `.props`, `.targets`, and `.slnx` coverage.
  - Decide whether classic `.sln` needs extension-owned treatment or is already
    correctly handled by Zed; avoid duplicate ownership.
  - Acceptance: supported project files have a documented language association
    and useful structure/highlighting.

## Milestone 3 — Razor semantic proof of concept

Goal: establish the smallest sustainable path to project-aware Razor semantics.
**Exists only if G0 was positive.**

- [ ] **M3.1 — Pin compatible Roslyn/Razor distribution**
  - Record the exact Roslyn server distribution, Razor cohost component, SDK
    requirements, argument values (as proven in M0.0), and supported runtime
    identifiers.
  - Never mix arbitrary Razor DLLs with an unrelated Roslyn version.
  - If G0 landed on "needs components not in the package", this item first
    covers acquiring, licensing, and repackaging those components, and must be
    re-estimated before work starts.
  - Acceptance: server startup is reproducible for each supported platform and
    reports a clear version/component mismatch.

- [ ] **M3.2 — Build the semantic capability probe**
  - Attach Razor to Roslyn using the `aspnetcorerazor` language identifier via
    the `language_ids` map from M1.1.
  - Record dynamic registrations and Razor-specific client requests/notifications.
  - Test C# completion, hover, diagnostics, navigation, references, rename,
    actions, semantic tokens, folding, and formatting separately.
  - Acceptance: an evidence table marking each capability as working, degraded,
    unsupported by Zed, or requiring a bridge/core change.

- [ ] **M3.3 — Propose the minimum upstream Zed capability**
  - Open a Zed GitHub Discussion with the probe, real user value, protocol
    trace, alternatives considered, test fixtures, and a minimal API/core design.
  - Note that zed#21133 (LSP protocol extensions in extensions) was closed as
    not planned; the proposal must argue why Razor is a different case rather
    than re-raising a settled one.
  - Obtain staff feedback before preparing a feature PR; sign the Zed CLA
    before submitting to `zed-industries/zed`. (This is a contribution to Zed
    core, and is unaffected by the fork decision — M3.3 and M4.2A concern Zed
    itself, not the extension repository.)
  - Acceptance: an explicit accept, revise, or decline outcome — or 30 days of
    silence, which counts as a decline for planning purposes and starts G2.

- [ ] **M3.4 — Publish experimental Razor semantics**
  - Ship as its own release, separate from M1.5, using the opt-in mechanism
    established in M0.4 with precise experimental wording.
  - Enable only capabilities proven safe; specifically withhold formatting and
    mapped edits unless virtual/projected document mapping is verified.
  - Acceptance: turning it off returns immediately to complete editing support
    with no data loss or configuration cleanup required.

## Milestone 4 — Architecture gate and stable semantics

Goal: choose a controlled, maintainable full-semantic implementation.

- [ ] **M4.1 — Run the 30-day decision gate (G2)**
  - Start the clock once M3.3 has a complete discussion/proof of concept, or
    once M3.3's 30-day silence timeout expires.
  - Choose the upstream route only with explicit Zed collaboration/acceptance.
  - Otherwise begin the proxy-process feasibility design in M4.2B.
  - The decision record must price both routes honestly. In particular, M4.2B
    is **not** the cheaper option: see its cost note below.
  - Acceptance: decision record identifies ownership, compatibility contract,
    maintenance burden, and a migration/fallback plan. "Stop at experimental"
    is a permitted outcome and must be considered explicitly.

- [ ] **M4.2A — Upstream route (conditional)**
  - Implement only the minimum accepted Zed capability for Razor-specific
    requests and projected documents.
  - Keep language registration, configuration, Roslyn distribution, fixtures,
    and documentation in this extension.
  - Acceptance: upstream tests and extension integration tests pass; no
    dependency on unreleased behavior is advertised as stable.

- [ ] **M4.2B — Proxy-process route (conditional)**
  - **Architecture, stated precisely:** the bridge cannot live inside the
    extension. It is a separate executable that `language_server_command`
    returns; it spawns Roslyn as a child and speaks LSP on both sides, owning
    virtual HTML documents and mapping requests, responses, ranges, edits,
    diagnostics, and lifecycle events.
  - Because Zed only opens real files, the proxy owns the virtual-document
    lifecycle outright — nothing leaks into ordinary editor workflows unless
    the proxy emits it. Range mapping must therefore be complete and lossless
    in the proxy; there is no editor-side safety net.
  - **Cost note for M4.1:** this means a second program, in a second language,
    built and shipped per runtime identifier through the same download-and-
    extract path `src/language_servers/nuget.rs` implements today, pinned
    against a Roslyn version whose Razor protocol is undocumented, maintained
    indefinitely. It roughly doubles the maintenance surface of this
    repository. The fork means this is purely this project's call to make —
    no maintainer has to be persuaded, and no one else absorbs the cost either.
  - Define failure isolation, logging/redaction, update policy, and a fast
    disable path that falls back to editing-only without a restart.
  - Acceptance: protocol conformance and range-mapping fixtures pass; killing
    the proxy degrades to editing-only rather than breaking the buffer.

- [ ] **M4.3 — Stabilize Razor semantics**
  - Make Roslyn Razor semantics the default only when the selected route passes
    all quality gates.
  - Enable Razor formatting only after correct formatting edits are proven.
  - Keep OmniSharp and `csharp-ls` visibly C#-only for semantic expectations.
  - Acceptance: release notes accurately distinguish stable behavior, optional
    integrations, and unsupported legacy cases.

## Quality gates

### How the matrix actually gets run

The matrix below is a 3-OS × 6-project-type × 10-operation space. One developer
on one machine cannot execute it by hand per release, so it is split:

- **Automated, every push (M1.0 harness + CI):** grammar/query snapshots over
  the M0.3a corpus on Linux; build of the extension crate on all three OSes.
- **Automated, per release:** server download and startup smoke on each
  supported runtime identifier; task definitions against M0.3b fixtures.
- **Manual, per release, checklist-driven:** the semantic-operation row and the
  lifecycle row, on the developer's primary OS, with the other two OSes covered
  by CI-executable smoke only.
- Anything in the matrix with no home in the three buckets above is either
  automated as part of the milestone that introduces it, or removed from the
  matrix. An unrunnable gate is not a gate.

### Functional matrix

| Dimension | Required coverage |
| --- | --- |
| SDK | .NET 8 LTS and newer; `global.json` selection/failure |
| OS | macOS (arm64/x64 where available), Linux, Windows |
| Projects | MVC, Razor Pages, Blazor Web App, Blazor WASM, RCL, multi-project, multi-target |
| Razor features | Directives, components/parameters, Tag Helpers, layouts/partials, page models, scoped CSS, JS interop, HTML/C#/CSS/JS embeddings |
| Semantic operations | Completion, hover, diagnostics, definition, references, rename, actions, semantic tokens, folding, formatting |
| Lifecycle | Cold open, restore/build, project reload, external edits, multiple worktrees, offline mode, stale/missing generated state, server mismatch |
| Failure | Unsupported SDK/project, server launch failure, protocol error, malformed template, disabled Roslyn, conflicting Razor extension |

### Release blockers

- Incorrect mapped edit, range, diagnostic, or navigation target.
- Formatting changes a location outside the intended Razor source range.
- Crash, unbounded memory growth, or material startup/large-solution regression.
- Silent server fallback, automatic build/restore, telemetry, or project-content
  upload.
- A regression in ordinary C# language/server behavior, as caught by the M1.0
  harness extended over the existing `csharp` queries.
- A language server starting for Razor buffers in a release that does not
  advertise Razor semantics.

## Out of scope

- A new debugger or debug adapter.
- Application generators/scaffolding.
- Cloud/deployment management.
- Guaranteed semantic support for classic ASP.NET/.NET Framework Razor.
- Replacement implementations of HTML, CSS, JavaScript, or TypeScript support.
