# Razor grammar audit and decision (M0.2, gate G1)

Status: **decided 2026-09-05**. Outcome recorded in `BACKLOG.md`.

## Candidates considered

There is no canonical `tree-sitter-razor`. The three candidates named in the
backlog were evaluated at the revisions below:

| Candidate | Kind | Licence | Verdict |
| --- | --- | --- | --- |
| [tris203/tree-sitter-razor](https://github.com/tris203/tree-sitter-razor) @ `d4664e409caaea12f73c9525484e3cf88b1cf718` | real grammar (embeds `tree-sitter-c-sharp`) | MIT © 2023 Tristan Knight, LICENSE file present, actively maintained (July 2026) | **chosen — forked** |
| [IbrahimSabriOrene/zed-razor-treesitter](https://github.com/IbrahimSabriOrene/zed-razor-treesitter) @ `ed8e4c6e9bb9749352fd6197457f2406f9290a88` | real grammar at repo root | package.json declares MIT but **no LICENSE file exists**; author field is template residue ("Your Name") | rejected — licence obligations cannot be verified; also fails the `@@`-escape core construct (45.77% of file in ERROR) |
| [NoisKung/razor-syntex-zed](https://github.com/NoisKung/razor-syntex-zed) @ `07d7334a8987a6323544c532c7127e3e010d7291` | **not a grammar** — queries over the stock `html` grammar | MIT | not a grammar candidate; it is the G1 "injection-only" else-branch shape, kept as a design reference only |

## Fitness test method

`scripts/grammar-fitness.py` parses the M0.3a corpus (`corpus/razor/`) with the
candidate grammar and reports, per file: presence of `ERROR`/`MISSING` nodes,
share of file bytes inside outermost `ERROR` spans, and (pathological half)
whether the error extends to end-of-file. Bands were fixed in the backlog
**before** running. The corpus at audit time: 14 well-formed + 10 pathological
files.

## Results at the pinned upstream revision (tris203, `d4664e4`)

- Well-formed half: **3/14 files affected (21.43%)**, worst single `ERROR`
  span **99.73%** of a file (`razor-comments.cshtml`) — auto-Reject by the
  >50% rule as-is.
- Pathological half: **4/10 (40%)** errors extend to EOF (gate: ≤20%).
- Upstream's own corpus: 78/79 (one pre-existing failure fixed by the fork,
  see below).
- Upstream's tests never exercise a `@* … *@` comment inside a braced razor
  block; that construct fails on upstream too (verified against pristine
  `d4664e4`).

All three well-formed failures were **specific, nameable constructs**:

1. A `@* … *@` comment as the first item of any braced razor block
   (`@code`, `@if`, `@{ … }`, `@foreach` body) derailed the whole file.
2. `@media`/`@supports`/… in `<style>` lexed as a Razor implicit expression;
   the selector prelude then derailed as a bogus explicit expression.
3. Parenthesized lambdas in attribute values (`@onclick="() => Fn()"`), plus
   explicit expressions in values (`@onclick="@(e => Save(e))"`), lost the
   `(`-ambiguity and broke neighbouring lines through GLR interaction.

## Fork decision and fixes (G1: fork)

**Decision: fork** `tris203/tree-sitter-razor` into a grammar repository of
its own, [joeizang/tree-sitter-razor](https://github.com/joeizang/tree-sitter-razor), pinned by commit from `extension.toml` exactly as the
`c_sharp` and `xml` grammars are. The project accepts maintaining this grammar
fork indefinitely (recorded as required by the backlog; on current evidence it
was always likely). Forking rather than referencing upstream keeps fixes off
upstream's review queue; keeping the grammar in its own repository rather than
vendored keeps this repository small (a vendored copy is ~48 MB, cloned by
every user at install), lets the grammar version independently of the
extension, and avoids a pin that can only be written after the merge it refers
to.

The fork revision in use is pinned in `extension.toml`.

Local modifications, all in `grammar.js` (documented in the file header):

1. **Root cause of the comment derailment was not the comment's lexer.** It
   was `razor_comment` being listed both in `extras` and as a `_node` member:
   at the start of a `repeat` (a fresh block body) the extra-path and the
   item-path fork and the structural parse starves. **Fix: removed
   `razor_comment` from `extras`** (it remains a first-class `_node`
   member, so it still parses everywhere a `_node` can appear). The comment
   text rule was left as upstream so C# comments still parse as children
   inside a razor comment, matching upstream's expected trees.
2. **`css_at_rule`** for CSS at-rules (`@media`, `@supports`, `@keyframes`,
   …) with a prelude token stopping at `{`/`;`. `@page` is deliberately
   **not** claimed — it is a Razor directive in `.cshtml` (and `@namespace`
   likewise: claiming it broke upstream's own Namespace test).
3. **`razor_attribute_value`** gained `prec.left(1)` and accepts
   `razor_explicit_expression`, so `@onclick="() => Fn()"` and
   `@onclick="@(e => Save(e))"` parse cleanly.

Fix 1 also fixes upstream's pre-existing failing test ("Namespace" initially
failed under fix 2 until `@namespace` was removed from the CSS list; the
pre-existing "Commented Code Block" interaction required fix 1 to be the
extras-removal form rather than a single-token comment body).

## Results after the fork fixes

- Upstream corpus: **79/79**.
- M0.3a corpus, well-formed half: **14/14 clean — 0% affected, 0% worst-file
  span, 0 core-construct failures → Adopt-band criteria met by the fork**.
- Pathological half: 3/10 outermost `ERROR` spans extend to EOF (30%;
  aspirational gate ≤20%). Of these:
  - `cascading-unterminated.cshtml` ends *inside* the unterminated `@{` block;
    an ERROR reaching EOF is the only correct outcome by construction.
  - `unterminated-if-block.cshtml`: every construct after the break —
    including the trailing `<p>` — parses to its correct node type inside the
    ERROR wrapper; only the outer span reaches EOF.
  - `broken-attribute.cshtml`: the unterminated attribute quote swallows the
    remainder; the trailing `<p>` is not recovered. **Known limit**, inherited
    from upstream (worse there: 65% of the file in one ERROR span), recorded
    for M1.2. A grammar-level fix (terminating quoted values at `>`) would
    break legitimate `>` inside attribute values and was rejected.

## Licence and attribution obligations

- The grammar is MIT; `LICENSE` is retained unmodified at the root of the
  grammar repository.
- The grammar repository's README carries a fork notice naming upstream, the pinned
  commit, and the local modifications; `grammar.js` carries the same in its
  header. Apache-2.0's "mark modifications" obligation is satisfied by these
  notices.
- If any upstream query file is later copied or adapted into
  `languages/razor/*.scm`, its header must name the upstream repository,
  revision, licence, and the local changes (tracked under M1.2).
- Fork hygiene for grammar updates: re-run `scripts/grammar-fitness.py` over
  the corpus and `tree-sitter test` in the grammar checkout after every
  upstream merge (D0.1) or grammar change; both are wired into CI (M1.0).
