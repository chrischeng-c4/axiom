#!/usr/bin/env python3.12
"""Grep-driven panic-boundary audit for mamba JIT/FFI entry points.

This is a test/tooling-only inventory for issue #1125. It does not try to prove
soundness statically; it surfaces the concrete places that merit manual review:

* `extern "C"` function definitions that may not unwind safely
* raw-address JIT/native callsites cast to `extern "C" fn`
* configured panic strategy (`panic = "abort"` vs the default unwind strategy)
* presence of generated `catch_unwind` wrapper support in `src/ffi/safety.rs`

The audit is intentionally grep-driven and dependency-free so it can run in
local dev loops and nightly jobs without requiring Cargo or Rust AST tooling.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
WORKSPACE_ROOT = MAMBA_DIR.parents[1]
SRC_ROOT = MAMBA_DIR / "src"
RUNTIME_ROOTS = (SRC_ROOT / "runtime", SRC_ROOT / "ffi")
ROOT_CARGO_TOML = WORKSPACE_ROOT / "Cargo.toml"
PROJECT_CARGO_TOML = MAMBA_DIR / "Cargo.toml"

EXTERN_FN_RE = re.compile(
    r'(?m)^(?P<indent>\s*)(?P<vis>pub\s+)?(?P<unsafe>unsafe\s+)?extern\s+"C"\s+fn\s+'
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\("
)
EXTERN_BLOCK_RE = re.compile(r'(?m)^(?P<indent>\s*)extern\s+"C"\s*\{')
PANIC_PROFILE_RE = re.compile(r'(?ms)^\[profile\.(?P<profile>[^\]]+)\](?P<body>.*?)(?=^\[|\Z)')
PANIC_VALUE_RE = re.compile(r'(?m)^\s*panic\s*=\s*"(?P<value>abort|unwind)"\s*$')

RISK_TOKENS = (
    "unwrap(",
    "expect(",
    "panic!(",
    "assert!(",
    "assert_eq!(",
    "assert_ne!(",
    "debug_assert!(",
    "unreachable!(",
)


@dataclass
class AuditEntry:
    kind: str
    path: str
    line: int
    symbol: str
    status: str
    reason: str
    risky_tokens: list[str]
    has_catch_unwind: bool
    snippet: str
    command: list[str]
    is_test_context: bool

    def to_json(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "path": self.path,
            "line": self.line,
            "symbol": self.symbol,
            "status": self.status,
            "reason": self.reason,
            "risky_tokens": self.risky_tokens,
            "has_catch_unwind": self.has_catch_unwind,
            "snippet": self.snippet,
            "command": self.command,
            "is_test_context": self.is_test_context,
        }


def repo_rel(path: Path) -> str:
    return path.relative_to(WORKSPACE_ROOT).as_posix()


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def surrounding_lines(text: str, offset: int, before: int = 30) -> list[str]:
    lines = text[:offset].splitlines()
    return lines[-before:]


def detect_test_context(path: Path, text: str, offset: int) -> bool:
    if "/tests/" in path.as_posix():
        return True
    window = "\n".join(surrounding_lines(text, offset))
    return (
        "#[cfg(test)]" in window
        or "#[test]" in window
        or " mod tests" in window
        or "fn test_" in window
        or "fn should_" in window
    )


def function_block(text: str, start: int) -> tuple[str, int] | None:
    open_brace = text.find("{", start)
    if open_brace == -1:
        return None
    lines = text[open_brace:].splitlines(keepends=True)
    depth = 0
    collected: list[str] = []
    saw_open = False
    consumed = 0
    for line in lines:
        collected.append(line)
        depth += line.count("{")
        depth -= line.count("}")
        consumed += len(line)
        if "{" in line:
            saw_open = True
        if saw_open and depth <= 0:
            return "".join(collected), open_brace + consumed
    return None


def extern_block_body(text: str, start: int) -> tuple[str, int] | None:
    return function_block(text, start)


def risky_tokens(body: str) -> list[str]:
    return [token for token in RISK_TOKENS if token in body]


def classify_extern_fn(body: str, *, is_test_context: bool) -> tuple[str, str]:
    tokens = risky_tokens(body)
    has_catch = "catch_unwind" in body
    if has_catch:
        return "guarded", "body contains catch_unwind"
    if tokens and not is_test_context:
        return "needs_boundary_review", "extern body contains panic-adjacent tokens without catch_unwind"
    if tokens:
        return "test_only_risky", "test-only extern body contains panic-adjacent tokens without catch_unwind"
    return "plain_extern", "extern body has no direct panic markers in this grep slice"


def grep_command(pattern: str, rel_path: str) -> list[str]:
    return ["rg", "-n", pattern, rel_path]


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def audit_file(path: Path, *, include_tests: bool) -> list[AuditEntry]:
    text = load_text(path)
    rel = repo_rel(path)
    entries: list[AuditEntry] = []
    for match in EXTERN_FN_RE.finditer(text):
        is_test = detect_test_context(path, text, match.start())
        if is_test and not include_tests:
            continue
        block = function_block(text, match.start())
        body = block[0] if block else ""
        tokens = risky_tokens(body)
        has_catch = "catch_unwind" in body
        status, reason = classify_extern_fn(body, is_test_context=is_test)
        entries.append(
            AuditEntry(
                kind="extern_fn",
                path=rel,
                line=line_number(text, match.start()),
                symbol=match.group("name"),
                status=status,
                reason=reason,
                risky_tokens=tokens,
                has_catch_unwind=has_catch,
                snippet=text[match.start(): text.find("\n", match.start())].strip(),
                command=grep_command(match.group("name"), rel),
                is_test_context=is_test,
            )
        )

    for match in EXTERN_BLOCK_RE.finditer(text):
        is_test = detect_test_context(path, text, match.start())
        if is_test and not include_tests:
            continue
        block = extern_block_body(text, match.start())
        body = block[0] if block else ""
        entries.append(
            AuditEntry(
                kind="extern_block",
                path=rel,
                line=line_number(text, match.start()),
                symbol="extern_block",
                status="ffi_import",
                reason="raw imported FFI block; unwind safety depends on the imported symbol",
                risky_tokens=[],
                has_catch_unwind=False,
                snippet=text[match.start(): text.find("\n", match.start())].strip(),
                command=grep_command('extern "C"', rel),
                is_test_context=is_test,
            )
        )

    for idx, raw in enumerate(text.splitlines(), start=1):
        if 'extern "C" fn' not in raw or "transmute" not in raw:
            continue
        offset = text.find(raw)
        is_test = detect_test_context(path, text, offset)
        if is_test and not include_tests:
            continue
        entries.append(
            AuditEntry(
                kind="jit_callsite",
                path=rel,
                line=idx,
                symbol=f"transmute_callsite_{idx}",
                status="jit_entry_callsite",
                reason="raw address cast to extern fn; callee panic policy must not unwind across this boundary",
                risky_tokens=[],
                has_catch_unwind=False,
                snippet=raw.strip(),
                command=grep_command("transmute", rel),
                is_test_context=is_test,
            )
        )
    return entries


def discover_entries(*, include_tests: bool) -> list[AuditEntry]:
    entries: list[AuditEntry] = []
    for root in RUNTIME_ROOTS:
        for path in sorted(root.rglob("*.rs")):
            entries.extend(audit_file(path, include_tests=include_tests))
    return entries


def panic_profiles() -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    for cargo in (ROOT_CARGO_TOML, PROJECT_CARGO_TOML):
        text = load_text(cargo)
        rel = repo_rel(cargo)
        for match in PANIC_PROFILE_RE.finditer(text):
            panic_match = PANIC_VALUE_RE.search(match.group("body"))
            items.append(
                {
                    "path": rel,
                    "profile": match.group("profile"),
                    "panic": panic_match.group("value") if panic_match else None,
                    "reason": "explicit" if panic_match else "implicit_default_unwind",
                }
            )
    return items


def wrapper_support() -> dict[str, Any]:
    path = SRC_ROOT / "ffi" / "safety.rs"
    text = load_text(path)
    rel = repo_rel(path)
    line = next(
        (
            idx
            for idx, raw in enumerate(text.splitlines(), start=1)
            if "catch_unwind" in raw
        ),
        None,
    )
    return {
        "path": rel,
        "line": line,
        "has_catch_unwind_codegen": "catch_unwind" in text,
        "command": grep_command("catch_unwind", rel),
    }


def summarize(entries: list[AuditEntry]) -> dict[str, Any]:
    counts: dict[str, int] = {}
    for entry in entries:
        counts[entry.status] = counts.get(entry.status, 0) + 1
    risky = [entry for entry in entries if entry.status in {"needs_boundary_review", "test_only_risky"}]
    return {
        "total_entries": len(entries),
        "extern_fn_count": sum(1 for entry in entries if entry.kind == "extern_fn"),
        "extern_block_count": sum(1 for entry in entries if entry.kind == "extern_block"),
        "jit_callsite_count": sum(1 for entry in entries if entry.kind == "jit_callsite"),
        "counts": counts,
        "risky_entry_count": len(risky),
    }


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inventory panic boundaries around mamba JIT/FFI entry points.",
    )
    parser.add_argument("--json", action="store_true", help="emit JSON")
    parser.add_argument(
        "--include-tests",
        action="store_true",
        help="include test-context extern functions and callsites",
    )
    parser.add_argument(
        "--fail-on-risk",
        action="store_true",
        help="exit 1 when risky extern definitions are found",
    )
    return parser.parse_args(argv)


def print_human(payload: dict[str, Any]) -> None:
    summary = payload["summary"]
    print(
        "summary:"
        f" extern_fn={summary['extern_fn_count']}"
        f" extern_block={summary['extern_block_count']}"
        f" jit_callsite={summary['jit_callsite_count']}"
        f" risky={summary['risky_entry_count']}"
    )
    print("panic_profiles:")
    for item in payload["panic_profiles"]:
        value = item["panic"] or "implicit_default_unwind"
        print(f"  - {item['path']} [{item['profile']}]: {value}")
    wrapper = payload["wrapper_support"]
    print(
        "ffi_wrapper_support:"
        f" {wrapper['path']}:{wrapper['line']} catch_unwind_codegen={wrapper['has_catch_unwind_codegen']}"
    )
    print("risky_entries:")
    risky = [
        entry for entry in payload["entries"]
        if entry["status"] in {"needs_boundary_review", "test_only_risky"}
    ]
    if not risky:
        print("  - none")
        return
    for entry in risky:
        tokens = ",".join(entry["risky_tokens"]) or "-"
        print(
            f"  - {entry['status']} {entry['path']}:{entry['line']} {entry['symbol']}"
            f" tokens={tokens} reason={entry['reason']}"
        )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    entries = discover_entries(include_tests=args.include_tests)
    summary = summarize(entries)
    payload = {
        "summary": summary,
        "panic_profiles": panic_profiles(),
        "wrapper_support": wrapper_support(),
        "entries": [entry.to_json() for entry in entries],
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print_human(payload)
    if args.fail_on_risk and summary["risky_entry_count"] > 0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
