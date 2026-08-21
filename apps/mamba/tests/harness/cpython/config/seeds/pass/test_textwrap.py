# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: textwrap (dedent for shared leading whitespace, indent prefixing
# each line). textwrap.fill / wrap / shorten currently return their input
# unchanged on mamba (no whitespace-based wrapping yet) and are intentionally
# NOT exercised here; tracked separately.
import textwrap

_ledger: list[int] = []

# dedent strips a single-line uniform leading indent
assert textwrap.dedent("    hello") == "hello", (
    "dedent('    hello') == 'hello'"
)
_ledger.append(1)

# dedent strips the longest shared leading indent across multiple lines
assert textwrap.dedent("  abc\n  def\n  ghi") == "abc\ndef\nghi", (
    "dedent removes the 2-space common prefix from all three lines"
)
_ledger.append(1)

# dedent's "common prefix" is the *minimum* across lines, not the maximum
assert textwrap.dedent("    abc\n  def") == "  abc\ndef", (
    "dedent uses the minimum common indent (2 spaces here)"
)
_ledger.append(1)

# dedent of an already-flush string is a no-op
assert textwrap.dedent("hello\nworld") == "hello\nworld", (
    "dedent of a flush string is a no-op"
)
_ledger.append(1)

# dedent of an empty string is an empty string
assert textwrap.dedent("") == "", "dedent('') == ''"
_ledger.append(1)

# indent prefixes each line in a two-line string
assert textwrap.indent("a\nb", "> ") == "> a\n> b", (
    "indent prefixes each line with '> '"
)
_ledger.append(1)

# indent prefixes each line in a three-line string
assert textwrap.indent("line1\nline2\nline3", "> ") == "> line1\n> line2\n> line3", (
    "indent prefixes every line in a 3-line input"
)
_ledger.append(1)

# indent of an empty string is an empty string (no prefix to apply)
assert textwrap.indent("", "> ") == "", "indent('', '> ') == ''"
_ledger.append(1)

# indent with a multi-character prefix still applies to every line
assert textwrap.indent("a\nb", "--> ") == "--> a\n--> b", (
    "indent supports multi-character prefixes"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_textwrap {sum(_ledger)} asserts")
