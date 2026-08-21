# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""slice: subscript syntax builds slice objects as keys (CPython 3.12)."""

# `obj[a:b:c]` desugars to obj.__getitem__/__setitem__ with a slice key.
# A class with only __setitem__ still receives a slice for extended-slice
# assignment.

seen = []


class Recorder:
    def __getitem__(self, key):
        return key

    def __setitem__(self, key, value):
        seen.append((key, value))


r = Recorder()

# Read subscripts hand back exactly the slice the syntax built.
assert r[1:2] == slice(1, 2)
assert r[1:2:3] == slice(1, 2, 3)
assert r[:] == slice(None, None, None)
assert r[::-1] == slice(None, None, -1)

# Write subscripts pass the same slice object to __setitem__.
r[1:2] = 42
assert seen == [(slice(1, 2), 42)]

# Bare-int subscripts are NOT slices.
assert r[5] == 5

print("subscript_key OK")
