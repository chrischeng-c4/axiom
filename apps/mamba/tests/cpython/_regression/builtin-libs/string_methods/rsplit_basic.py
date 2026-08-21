# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: str.rsplit was unimplemented — the method dispatcher fell
# through to the unknown-attribute branch and returned None.

s = "a.b.c.d"
print(s.rsplit("."))
print(s.rsplit(".", 1))
print(s.rsplit(".", 2))
print(s.rsplit(".", 3))
print(s.rsplit(".", 10))
print(s.rsplit(".", 0))

# Whitespace sep (None)
print("  a   b  c  ".rsplit())
print("  a   b  c  ".rsplit(None, 1))
print("one two three four".rsplit(None, 2))

# Edge cases
print("".rsplit("."))
print("abc".rsplit(","))
print("abc".rsplit(",", 0))

# Multi-char separator
print("a--b--c".rsplit("--"))
print("a--b--c".rsplit("--", 1))
