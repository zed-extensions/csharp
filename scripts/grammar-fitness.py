#!/usr/bin/env python3
"""Grammar fitness test (M0.2) — run a tree-sitter grammar over the Razor corpus.

Measures, per file: presence of ERROR/MISSING nodes, share of file bytes
covered by ERROR spans, and — for the pathological half — whether the error
extends to end-of-file and whether content after the break still parses to
correct node types.

Usage:
  grammar-fitness.py <grammar-dir> <corpus-dir> [--json OUT]

The grammar-dir must already have `tree-sitter generate` applied (or a
committed src/parser.c) so `tree-sitter parse` works. Bands (well-formed half):

  Adopt as-is   <=5% of files with any ERROR/MISSING, worst-file span <=10%,
                zero core-construct failures
  Fork and fix  >5% and <=25% of files, <=3 core-construct failures
  Reject        >25% of files, >3 core-construct failures, or any single ERROR
                spanning >50% of a file

Pathological half gate is containment: ERROR extends to EOF in <=20% of files.
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

# File -> core constructs it exercises (backlog M0.2 list).
CORE_CONSTRUCTS = {
    "pages-implicit-expressions.cshtml": ["implicit expression", "@foreach", "@if"],
    "explicit-expressions.cshtml": ["explicit expression"],
    "code-block.razor": ["@code block", "@onclick", "@inject", "@using", "@page", "@parameter"],
    "functions-block.razor": ["@functions block"],
    "directives-and-control-flow.cshtml": ["@page", "@model", "@using", "@inject", "@inherits", "@attribute", "@switch", "@foreach", "@if"],
    "component-parameters.razor": ["component with parameters"],
    "tag-helpers.cshtml": ["tag helper (asp-for)"],
    "bind-and-events.razor": ["@bind", "@onclick"],
    "layouts-and-sections.cshtml": ["@section", "layout/partial"],
    "razor-comments.cshtml": ["razor comment"],
    "at-escapes-and-email.cshtml": ["@@ escape", "email in text"],
    "style-media-rule.razor": ["@media in <style>", "expression in CSS"],
    "script-block.cshtml": ["@ in <script>"],
    "mixed-statements.cshtml": ["@for", "@try/catch", "layout"],
}

NODE = re.compile(r"\(([A-Za-z_][A-Za-z0-9_]*)")
SPAN = re.compile(r"\[(\d+), (\d+)\] - \[(\d+), (\d+)\]")


def parse_file(grammar_dir: Path, path: Path) -> tuple[str, int]:
    proc = subprocess.run(
        ["tree-sitter", "parse", str(path.resolve())],
        cwd=grammar_dir,
        capture_output=True,
        text=True,
    )
    return proc.stdout + proc.stderr, proc.returncode


def error_spans(tree: str) -> list[tuple[int, int, int, int]]:
    """Return (start_row, start_col, end_row, end_col) for every outermost
    ERROR node in a tree S-expression dump.

    Tree-sitter nests children inside ERROR nodes (each repeating a range),
    and the CLI's trailing summary line repeats the widest span, so nested and
    duplicated spans are dropped: only spans not contained in another span are
    returned.
    """
    raw: list[tuple[int, int, int, int]] = []
    for m in re.finditer(r"\(ERROR", tree):
        r = SPAN.search(tree, m.start())
        if r:
            raw.append(tuple(int(g) for g in r.groups()))
    outer = [
        s
        for s in raw
        if not any(s is not t and _contains(t, s) for t in raw)
    ]
    return outer


def _contains(
    big: tuple[int, int, int, int], small: tuple[int, int, int, int]
) -> bool:
    return (big[0], big[1]) <= (small[0], small[1]) and (small[2], small[3]) <= (
        big[2],
        big[3],
    )


def has_missing(tree: str) -> bool:
    return "(MISSING" in tree


def col_bytes(lines: list[str], row: int, col: int) -> int:
    # CLI reports byte columns; clamp defensively.
    if row >= len(lines):
        return 0
    return min(col, len(lines[row].encode()))


def span_bytes(lines: list[str], span: tuple[int, int, int, int]) -> int:
    (r1, c1, r2, c2) = span
    total = 0
    if r1 == r2:
        total = col_bytes(lines, r2, c2) - col_bytes(lines, r1, c1)
    else:
        total = len(lines[r1].encode()) - col_bytes(lines, r1, c1)
        for r in range(r1 + 1, r2):
            total += len(lines[r].encode()) + 1
        total += col_bytes(lines, r2, c2)
    return max(total, 0)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("grammar_dir")
    ap.add_argument("corpus_dir")
    ap.add_argument("--json", dest="json_out")
    args = ap.parse_args()

    grammar_dir = Path(args.grammar_dir)
    corpus = Path(args.corpus_dir)
    files = sorted(corpus.rglob("*.razor")) + sorted(corpus.rglob("*.cshtml"))

    report = {"grammar": str(grammar_dir), "corpus": str(corpus), "files": {}}
    wf_affected = 0
    wf_total = 0
    worst_span = 0.0
    cc_failures = 0
    eof_extension = 0
    path_total = 0

    for path in files:
        rel = path.relative_to(corpus).as_posix()
        text = path.read_bytes()
        n_bytes = len(text)
        lines = text.decode("utf-8", "replace").splitlines()
        out, rc = parse_file(grammar_dir, path)
        # The CLI's trailing summary line (`<path>\tParse: ...`) repeats the
        # widest error span; drop it so only tree output is scanned.
        tree_only = out.split(f"\tParse:", 1)[0]
        spans = error_spans(tree_only)
        covered = sum(span_bytes(lines, s) for s in spans)
        pct = covered / n_bytes * 100 if n_bytes else 0.0
        last_line = len(lines) - 1
        last_len = len(lines[last_line].encode()) if lines else 0
        to_eof = bool(spans) and all(
            (s[2], s[3]) >= (last_line, max(last_len - 1, 0)) for s in spans
        )
        entry = {
            "returncode": rc,
            "error_nodes": len(spans),
            "missing_nodes": has_missing(out),
            "error_bytes": covered,
            "error_pct": round(pct, 2),
            "error_reaches_eof": to_eof,
        }
        pathological = "pathological" in rel
        if pathological:
            path_total += 1
            if to_eof:
                eof_extension += 1
        else:
            wf_total += 1
            if spans or entry["missing_nodes"]:
                wf_affected += 1
            worst_span = max(worst_span, pct)
        report["files"][rel] = entry

    report["summary"] = {
        "well_formed_total": wf_total,
        "well_formed_affected": wf_affected,
        "well_formed_affected_pct": round(wf_affected / wf_total * 100, 2) if wf_total else 0,
        "worst_file_error_pct": round(worst_span, 2),
        "pathological_total": path_total,
        "pathological_eof_extension": eof_extension,
        "pathological_eof_pct": round(eof_extension / path_total * 100, 2) if path_total else 0,
    }

    # Core-construct verdicts are recorded by the auditor in
    # docs/razor-grammar-audit.md; the tree dump per file is the evidence.
    if args.json_out:
        Path(args.json_out).write_text(json.dumps(report, indent=2))
    print(json.dumps(report["summary"], indent=2))
    for rel, entry in report["files"].items():
        flag = "EOF" if entry["error_reaches_eof"] else ("err" if entry["error_nodes"] or entry["missing_nodes"] else "ok ")
        print(f"  {flag}  {entry['error_pct']:6.2f}%  {rel}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
