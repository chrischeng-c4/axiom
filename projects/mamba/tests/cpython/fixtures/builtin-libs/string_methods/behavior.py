# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/string_methods: behavior asserts (CPython 3.12 oracle)."""

# Return-type contracts: the search/predicate/transform families return
# the types their docs promise.
assert isinstance("abc".find("b"), int)
assert isinstance("abc".isalpha(), bool)
assert isinstance("abc".upper(), str)
assert isinstance("a,b".split(","), list)
assert isinstance("a,b".partition(","), tuple)
assert isinstance("abc".encode(), bytes)

# split / partition return-shape invariants.
assert "a,b,c".partition(",") == ("a", ",", "b,c")
assert "abc".partition(",") == ("abc", "", "")          # sep absent
assert "a,b,c".rpartition(",") == ("a,b", ",", "c")
assert len("a b c".split()) == 3
assert "  a  b  ".split() == ["a", "b"]                  # runs collapse
assert "a,,b".split(",") == ["a", "", "b"]              # empties kept

# Round-trip: join is the inverse of split on a fixed separator.
parts = "x|y|z".split("|")
assert "|".join(parts) == "x|y|z"

# find returns -1 (not an exception); index raises on miss.
assert "abc".find("z") == -1
raised = False
try:
    "abc".index("z")
except ValueError:
    raised = True
assert raised

# count is non-overlapping.
assert "aaaa".count("aa") == 2

# Strings are immutable: methods return new objects, original unchanged.
s = "Hello"
assert s.lower() == "hello"
assert s == "Hello"

# Empty-string corner cases.
assert "".split() == []
assert "".splitlines() == []
assert "".join([]) == ""
assert len("") == 0

print("behavior OK")
