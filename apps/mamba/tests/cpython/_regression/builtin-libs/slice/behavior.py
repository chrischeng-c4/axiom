# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/slice: behavior asserts (CPython 3.12 oracle)."""

# --- construction: arg count decides which fields are set ---
# Single arg fills stop; start/step default to None.
s = slice(1)
assert s.start is None and s.stop == 1 and s.step is None
# Two args fill start and stop.
s = slice(1, 2)
assert s.start == 1 and s.stop == 2 and s.step is None
# Three args fill all.
s = slice(1, 2, 3)
assert s.start == 1 and s.stop == 2 and s.step == 3
# Members may be arbitrary objects, not just ints.
obj = object()
assert slice(obj).stop is obj

# --- repr round-trips through the constructor form ---
assert repr(slice(1, 2, 3)) == "slice(1, 2, 3)"
assert repr(slice(None)) == "slice(None, None, None)"

# --- equality: field-wise, never equal to other types ---
assert slice(1, 2, 3) == slice(1, 2, 3)
assert slice(1, 2, 3) != slice(1, 2, 4)
assert slice(1, 2, 3) != None
assert slice(1, 2, 3) != (1, 2, 3)
assert slice(1, 2, 3) != ""

# --- ordering: slices compare like their (start, stop, step) tuples ---
assert slice(0, 5) < slice(0, 10)
assert slice(0, 10) > slice(0, 5)
assert slice(1, 2, 3) <= slice(1, 2, 3)
assert slice(1, 2, 3) >= slice(1, 2, 3)

# --- hash: equal slices hash equal; hash(s) == s.__hash__() ---
assert hash(slice(1, 2, 3)) == slice(1, 2, 3).__hash__()
assert hash(slice(1, 2, 3)) == hash(slice(1, 2, 3))

print("behavior OK")
