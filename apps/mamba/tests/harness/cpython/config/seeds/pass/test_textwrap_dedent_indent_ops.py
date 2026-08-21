# Operational AssertionPass seed for the `textwrap` module — the
# stdlib text manipulation utilities used by CLI help-text builders,
# docstring formatters, code-pretty-printers, and any code that
# strips / re-indents multi-line string literals. Surface focuses on
# the matching subset between mamba and CPython: `dedent` (strip
# common leading whitespace) and `indent` (prepend a prefix to each
# line). Mamba's `fill` / `wrap` / `shorten` don't actually
# break / shorten the input string (they return the original
# unchanged regardless of `width`), so those probes are left to a
# spec fixture and not exercised here. Mamba's `indent` also drops
# the trailing newline for inputs that end with `\n` (mamba returns
# `'Xa'` for `indent('a\n', 'X')`, CPython returns `'Xa\n'`), so
# trailing-newline cases are also excluded. No fixture coverage
# yet for textwrap.
#
# Surface (the matching subset):
#   • textwrap.dedent(text: str) → str
#       — strips the longest common leading whitespace prefix from
#         every non-empty line;
#       — `dedent("") == ""`;
#       — `dedent("a\nb\nc") == "a\nb\nc"` (no common prefix to strip);
#       — `dedent("  a\n  b") == "a\nb"`;
#       — mixed indents reduce to the minimum: dedent of
#         `"  a\n    b\n  c"` → `"a\n  b\nc"`;
#       — empty lines are preserved as-is;
#   • textwrap.indent(text: str, prefix: str) → str
#       — prepends `prefix` to the start of every non-empty line;
#       — `indent("", "X") == ""`;
#       — `indent("a", "X") == "Xa"`;
#       — `indent("a\nb", "X") == "Xa\nXb"`;
#       — multi-character prefixes work (e.g. `"> "`, `"// "`,
#         four-space `"    "`).
import textwrap
_ledger: list[int] = []

# dedent — empty input
assert textwrap.dedent("") == ""; _ledger.append(1)

# dedent — no common prefix
assert textwrap.dedent("a\nb\nc") == "a\nb\nc"; _ledger.append(1)
assert textwrap.dedent("hello") == "hello"; _ledger.append(1)

# dedent — uniform leading whitespace
assert textwrap.dedent("  a\n  b\n  c") == "a\nb\nc"; _ledger.append(1)
assert textwrap.dedent("    hello\n    world") == "hello\nworld"; _ledger.append(1)
assert textwrap.dedent("        a\n        b\n        c") == "a\nb\nc"; _ledger.append(1)

# dedent — mixed indents reduce to the minimum
assert textwrap.dedent("  a\n    b\n  c") == "a\n  b\nc"; _ledger.append(1)
assert textwrap.dedent("    a\n        b") == "a\n    b"; _ledger.append(1)

# dedent — single-line variants
assert textwrap.dedent("    one line") == "one line"; _ledger.append(1)
assert textwrap.dedent("        single") == "single"; _ledger.append(1)

# dedent — tab-only indent
assert textwrap.dedent("\ta\n\tb") == "a\nb"; _ledger.append(1)

# dedent — indent followed by empty line
assert textwrap.dedent("    \n    abc") == "\nabc"; _ledger.append(1)

# indent — empty input
assert textwrap.indent("", "X") == ""; _ledger.append(1)
assert textwrap.indent("", "    ") == ""; _ledger.append(1)
assert textwrap.indent("", "> ") == ""; _ledger.append(1)

# indent — single line (no trailing newline)
assert textwrap.indent("a", "X") == "Xa"; _ledger.append(1)
assert textwrap.indent("hello", "    ") == "    hello"; _ledger.append(1)
assert textwrap.indent("foo", "> ") == "> foo"; _ledger.append(1)

# indent — multi-line (no trailing newline)
assert textwrap.indent("a\nb", "X") == "Xa\nXb"; _ledger.append(1)
assert textwrap.indent("a\nb\nc", "X") == "Xa\nXb\nXc"; _ledger.append(1)
assert textwrap.indent("a\nb\nc", "    ") == "    a\n    b\n    c"; _ledger.append(1)
assert textwrap.indent("hello\nworld", "// ") == "// hello\n// world"; _ledger.append(1)
assert textwrap.indent("line1\nline2", "> ") == "> line1\n> line2"; _ledger.append(1)

# indent — multi-character / mixed prefixes
assert textwrap.indent("a\nb", "## ") == "## a\n## b"; _ledger.append(1)
assert textwrap.indent("a\nb", "[INFO] ") == "[INFO] a\n[INFO] b"; _ledger.append(1)
assert textwrap.indent("a\nb", "  -> ") == "  -> a\n  -> b"; _ledger.append(1)

# Return-type discipline — both always return str
assert isinstance(textwrap.dedent("  a"), str); _ledger.append(1)
assert isinstance(textwrap.dedent(""), str); _ledger.append(1)
assert isinstance(textwrap.indent("a", "X"), str); _ledger.append(1)
assert isinstance(textwrap.indent("", "X"), str); _ledger.append(1)

# Idempotence — calling twice returns same result
assert textwrap.dedent("  a\n  b") == textwrap.dedent("  a\n  b"); _ledger.append(1)
assert textwrap.indent("a\nb", "X") == textwrap.indent("a\nb", "X"); _ledger.append(1)

# dedent is idempotent on already-dedented input
_dedented = textwrap.dedent("    a\n    b")
assert textwrap.dedent(_dedented) == _dedented; _ledger.append(1)

# Module-level attribute discipline
assert hasattr(textwrap, "dedent"); _ledger.append(1)
assert hasattr(textwrap, "indent"); _ledger.append(1)
assert hasattr(textwrap, "fill"); _ledger.append(1)
assert hasattr(textwrap, "wrap"); _ledger.append(1)
assert hasattr(textwrap, "shorten"); _ledger.append(1)
assert callable(textwrap.dedent); _ledger.append(1)
assert callable(textwrap.indent); _ledger.append(1)
assert callable(textwrap.fill); _ledger.append(1)
assert callable(textwrap.wrap); _ledger.append(1)
assert callable(textwrap.shorten); _ledger.append(1)

# Length invariants — dedent never grows the string
for _s in ["", "a", "  a", "    a\n    b", "  a\n    b\n  c"]:
    assert len(textwrap.dedent(_s)) <= len(_s); _ledger.append(1)

# Length invariants — indent never shrinks the string
for _s, _p in [("a", "X"), ("a\nb", "X"), ("hello\nworld", "// ")]:
    assert len(textwrap.indent(_s, _p)) >= len(_s); _ledger.append(1)

# dedent on string with no whitespace — returns unchanged
assert textwrap.dedent("hello") == "hello"; _ledger.append(1)
assert textwrap.dedent("abc\ndef") == "abc\ndef"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_textwrap_dedent_indent_ops {sum(_ledger)} asserts")
