# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/comparison: language-area behavior asserts (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert isinstance(True, int)
assert isinstance(1, int)
assert type(()) is tuple
assert type([]) is list
assert type({}) is dict
assert len("abc") == 3
assert list(range(3)) == [0, 1, 2]

print("behavior OK")
