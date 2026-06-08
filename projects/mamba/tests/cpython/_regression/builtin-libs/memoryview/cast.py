# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/memoryview: cast / reinterpretation behavior (CPython 3.12)."""

import array

# Cast bytes to a 2-byte unsigned format reinterprets the elements.
raw = memoryview(b"\x01\x00\x02\x00")
shorts = raw.cast("H")
assert shorts.tolist() == [1, 2]
assert shorts.format == "H"
assert shorts.itemsize == 2
assert shorts.shape == (2,)

# Cast back down to bytes recovers the original layout.
back = shorts.cast("B")
assert back.tobytes() == b"\x01\x00\x02\x00"
assert back.format == "B"

# Multi-dimensional cast reshapes a flat buffer.
flat = memoryview(bytearray(6)).cast("B", shape=[2, 3])
assert flat.ndim == 2
assert flat.shape == (2, 3)

# Cast chains preserve the element format through nested views
# (regression: a sub-view and a memoryview-of-a-view keep 'H').
a = array.array("H", [256, 256, 256, 256])
base = memoryview(a)
as_bytes = base.cast("B")
as_shorts = as_bytes.cast("H")
sub = as_shorts[0:2]
nested = memoryview(as_shorts)
del as_shorts
assert sub[0] == 256
assert nested[0] == 256
assert sub.format == "H"
assert nested.format == "H"

# Re-casting the parent does not disturb the existing child views.
_ = as_bytes.cast("I")
assert sub[0] == 256
assert nested[0] == 256
assert sub.format == "H"
assert nested.format == "H"

# Assigning a whole array-backed view writes through element-wise.
nums = array.array("i", range(10))
view = memoryview(nums)
view[:] = array.array("i", range(9, -1, -1))
assert nums.tolist() == [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]

print("cast OK")
