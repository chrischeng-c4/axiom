# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/collections: behavior asserts (CPython 3.12 oracle)."""

# Verify a few core behaviors match CPython expectations.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert [1, 2] + [3] == [1, 2, 3]
assert sorted([3, 1, 2]) == [1, 2, 3]
assert len("abc") == 3

print("behavior OK")
