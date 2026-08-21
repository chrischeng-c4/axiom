#!/usr/bin/env python3.12
"""Generate ④ Safety wall: the error-leak matrix = builtin exceptions × secret classes.

Each cell = one fixture that triggers a builtin exception and asserts its error
message leaks no secret of a given class — a filesystem path, a memory address,
an environment value, or a source snippet (mamba .rs/.py internals). mamba's
runtime must never surface internals in an error message; a red cell (LEAK) marks
exactly a message the keep must sanitize.

228 cells = 57 builtin (non-Warning) exceptions × 4 secret classes.

    python3.12 safety_wall_gen.py --dry-run
    python3.12 safety_wall_gen.py --write
"""

from __future__ import annotations

import argparse
import builtins
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wall_gen_core import PEP723Header  # noqa: E402

MAMBA_DIR = Path(__file__).resolve().parents[4]
OUT_DIR = MAMBA_DIR / "tests" / "cpython" / "security-matrix"

# secret class -> (human description, regex that signals an internal leak)
SECRET_CLASSES = {
    "path": ("filesystem path", r"/(?:Users|home|private|tmp|var|opt|usr)/\S+"),
    "addr": ("memory address", r"0x[0-9a-fA-F]{6,}"),
    "env": ("environment value", r"(?:SECRET|TOKEN|API_KEY|PASSWORD|AWS_)\w*"),
    "source": ("source snippet", r"\.rs:\d+|/src/|File \"[^\"]+\", line \d+"),
}

# natural triggers for common exceptions; the rest fall back to a direct raise.
TRIGGERS = {
    "ValueError": 'int("not_a_number")',
    "TypeError": "len(42)",
    "KeyError": 'dict()["missing_key"]',
    "IndexError": "[][0]",
    "AttributeError": "(42).no_such_attribute",
    "ZeroDivisionError": "1 / 0",
    "NameError": "undefined_name_xyz",
    "FileNotFoundError": 'open("/no/such/path/canary")',
    "ModuleNotFoundError": "__import__('nonexistent_module_xyz')",
    "OverflowError": "__import__('math').exp(10000)",
    "UnicodeDecodeError": 'b"\\xff\\xfe".decode("utf-8")',
    "StopIteration": "next(iter([]))",
    "AssertionError": "_assert_false()",
    "RuntimeError": 'raise RuntimeError("runtime condition")',
}


def exceptions() -> list[str]:
    return sorted(
        n for n in dir(builtins)
        if isinstance(getattr(builtins, n), type)
        and issubclass(getattr(builtins, n), BaseException)
        and not issubclass(getattr(builtins, n), Warning)
    )


def render(exc: str, secret: str) -> tuple[str, str]:
    desc, pattern = SECRET_CLASSES[secret]
    trigger = TRIGGERS.get(exc, f'raise {exc}("error condition")')
    helper = "def _assert_false():\n    assert False\n\n" if trigger == "_assert_false()" else ""
    case = f"{exc}__{secret}"
    header = PEP723Header(
        bucket="security-matrix", lib=secret, dimension="security", case=case,
        subject=f"{exc} message must not leak {desc}", kind="semantic",
        xfail=f"error-message leak check; mamba must not surface {desc} in {exc} messages",
        mem_carveout="", source="", status="filled",
    ).render()
    text = header + f'''"""Safety wall ({secret}): {exc} message must not leak a {desc}.

Triggers {exc} and scans its message for an internal {desc}. mamba must keep
error messages free of internals; LEAK marks a message the keep must sanitize."""

import re

PATTERN = r"{pattern}"
{helper}try:
    {trigger}
    raised = None
except BaseException as e:
    raised = e

msg = "" if raised is None else (str(raised) + " " + repr(raised))
print("LEAK:{secret}" if re.search(PATTERN, msg) else "safe:{secret}")
'''
    return f"{secret}/{case}.py", text


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args()

    excs = exceptions()
    cells = [(e, s) for s in SECRET_CLASSES for e in excs]

    if args.dry_run or not args.write:
        print(f"error-leak matrix cells: {len(cells)} "
              f"({len(excs)} exceptions × {len(SECRET_CLASSES)} secret classes)")
        print("  exceptions:", ", ".join(excs[:10]), "...")
        print("  secret classes:", ", ".join(SECRET_CLASSES))
        print(f"  natural triggers: {len(TRIGGERS)} (rest = direct raise)")
        return 0

    written = 0
    for exc, secret in cells:
        rel, text = render(exc, secret)
        path = OUT_DIR / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        written += 1
    print(f"wrote {written} error-leak matrix cells under {OUT_DIR}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
