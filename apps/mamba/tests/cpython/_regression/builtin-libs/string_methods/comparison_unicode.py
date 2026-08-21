# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""str comparison / iteration / repr across Unicode planes (CPython 3.12)."""

# --- equality and ordering --------------------------------------------
assert "abc" == "abc"
assert "abc" < "abcd"            # prefix is smaller
assert "abcd" > "abc"
assert "Abc" < "abc"             # 'A'(65) < 'a'(97)

# Ordering is by code point, so it spans Unicode planes monotonically.
ascii_s = "a" * 4               # U+0061
latin = "\x80" * 4              # U+0080
bmp = "Ā" * 4                  # U+0100
astral = "\U00100000" * 4       # plane 16
assert ascii_s < latin < bmp < astral
assert astral >= bmp >= latin >= ascii_s
assert not ascii_s >= latin
print("ordering:", sorted(["b", "A", "Ā", "a"]))

# --- str is never equal to bytes --------------------------------------
assert ("abc" == b"abc") is False
assert ("abc" != b"abc") is True
assert ("abc" == bytearray(b"abc")) is False
print("str vs bytes:", "x" == b"x")

# --- iterating yields whole code points, including astral ones --------
it = iter("aᄑ𐀂")
assert next(it) == "a"
assert next(it) == "ᄑ"
assert next(it) == "𐀂"          # one astral char, not a surrogate pair
raised = False
try:
    next(it)
except StopIteration:
    raised = True
assert raised
assert list("a𐀂b") == ["a", "𐀂", "b"]
assert len("a𐀂b") == 3
print("iteration:", list("héllo"))

# --- repr escapes non-printable but keeps printable Unicode -----------
assert repr("abc") == "'abc'"
assert repr("a'b") == '"a\'b"'                 # switches quote style
assert repr("a\nb") == "'a\\nb'"               # control char escaped
assert repr("café") == "'café'"                # printable Unicode kept
assert repr("͸") == "'\\u0378'"           # unassigned -> escaped
print("repr:", repr("tab\there"))

print("comparison_unicode OK")
