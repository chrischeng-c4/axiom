# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""string.Formatter: format(), conversions, lookups (CPython 3.12 oracle)."""

import string

fmt = string.Formatter()

# --- basic positional / keyword ---------------------------------------
assert fmt.format("foo") == "foo"
assert fmt.format("foo{0}", "bar") == "foobar"
assert fmt.format("foo{1}{0}-{1}", "bar", 6) == "foo6bar-6"
assert fmt.format("-{arg}-", arg="test") == "-test-"
print("basic:", fmt.format("{0}-{name}", "x", name="y"))

# --- conversion specifiers (!r, !s, !a) -------------------------------
assert fmt.format("-{arg!r}-", arg="test") == "-'test'-"
assert fmt.format("{0!s}", "test") == "test"
assert fmt.format("{0!a}", 42) == "42"
assert fmt.format("{0!a}", chr(255)) == "'\\xff'"   # ascii() escapes
assert fmt.format("{0!a}", chr(256)) == "'\\u0100'"

# An unknown conversion is a ValueError.
raised = False
try:
    fmt.format("{0!h}", "test")
except ValueError:
    raised = True
assert raised

# --- index lookup (subscript on the argument) -------------------------
lookup = ["eggs", "and", "spam"]
assert fmt.format("{0[2]}{0[0]}", lookup) == "spameggs"
raised = False
try:
    fmt.format("{0[2]}", [])
except IndexError:
    raised = True
assert raised

# --- name lookup (attribute on the argument) --------------------------
class AnyAttr:
    def __getattr__(self, attr):
        return attr

assert fmt.format("{0.lumber}{0.jack}", AnyAttr()) == "lumberjack"
print("lookups:", fmt.format("{0[1]}.{0[2]}", lookup))

# --- auto vs manual numbering cannot be mixed -------------------------
assert fmt.format("a{}b{}c", 0, 1) == "a0b1c"
for bad in ("{}{1}", "{1}{}"):
    raised = False
    try:
        fmt.format(bad, 1, 2)
    except ValueError:
        raised = True
    assert raised, bad

print("string_formatter OK")
