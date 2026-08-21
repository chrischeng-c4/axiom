# Operational AssertionPass seed for `textwrap.dedent` + `textwrap.indent`.
# Surface: dedent strips common leading whitespace; indent prefixes
# each line with a prefix string. fill/wrap/shorten intentionally
# omitted — they currently no-op on mamba and that gap is tracked
# elsewhere.
# Companion to stub/test_textwrap.py — vendored unittest seed.
import textwrap
_ledger: list[int] = []
# dedent — strips the common leading whitespace
assert textwrap.dedent("    a\n    b\n    c") == "a\nb\nc"; _ledger.append(1)
assert textwrap.dedent("  x\n    y") == "x\n  y"; _ledger.append(1)
# No common prefix → unchanged
assert textwrap.dedent("abc\ndef") == "abc\ndef"; _ledger.append(1)
# indent — prefixes every line
assert textwrap.indent("line1\nline2", ">>") == ">>line1\n>>line2"; _ledger.append(1)
assert textwrap.indent("a\nb\nc", "* ") == "* a\n* b\n* c"; _ledger.append(1)
# Empty prefix is identity
assert textwrap.indent("hello", "") == "hello"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_textwrap_ops {sum(_ledger)} asserts")
