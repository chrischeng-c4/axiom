# Operational AssertionPass seed for stepped-slice surfaces on lists,
# strings, and tuples. Surface: positive step `[::2]` keeps every
# second element; alternate step `[::3]`; reverse via negative step
# `[::-1]`; explicit start:end:step slices on both ascending and
# descending direction; start-only `[a:]`, end-only `[:b]`; negative-
# index start `[-n:]` and negative end `[:-n]` and `[-a:-b]`; empty
# results — both empty-forward `[5:2]` and over-the-end `[100:]`;
# negative start that's clipped to 0 `[-100:3]`; string slice with
# step and reverse; tuple slice with step preserving tuple type.
_ledger: list[int] = []

a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Positive step
assert a[::2] == [0, 2, 4, 6, 8]; _ledger.append(1)
assert a[::3] == [0, 3, 6, 9]; _ledger.append(1)
assert a[::5] == [0, 5]; _ledger.append(1)

# Reverse with step -1
assert a[::-1] == [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]; _ledger.append(1)

# Explicit start:end:step (positive step)
assert a[1:8:2] == [1, 3, 5, 7]; _ledger.append(1)
assert a[0:6:2] == [0, 2, 4]; _ledger.append(1)
assert a[2:9:3] == [2, 5, 8]; _ledger.append(1)

# Explicit start:end:step (negative step) — walks high → low
assert a[8:1:-2] == [8, 6, 4, 2]; _ledger.append(1)
assert a[9:0:-3] == [9, 6, 3]; _ledger.append(1)

# Start-only and end-only slices
assert a[3:] == [3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)
assert a[:3] == [0, 1, 2]; _ledger.append(1)
assert a[:] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)

# Negative indices
assert a[-3:] == [7, 8, 9]; _ledger.append(1)
assert a[:-3] == [0, 1, 2, 3, 4, 5, 6]; _ledger.append(1)
assert a[-5:-2] == [5, 6, 7]; _ledger.append(1)
assert a[-1:] == [9]; _ledger.append(1)
assert a[:-1] == [0, 1, 2, 3, 4, 5, 6, 7, 8]; _ledger.append(1)

# Empty-result edge cases
# Forward slice with start > end → empty
assert a[5:2] == []; _ledger.append(1)
# Backward range with default (positive) step → empty
assert a[2:5:-1] == []; _ledger.append(1)
# Start past the end → empty
assert a[100:] == []; _ledger.append(1)
# End before the start (positive indices) → empty
assert a[7:3] == []; _ledger.append(1)

# Negative start clipped to 0 (we go past the front)
assert a[-100:3] == [0, 1, 2]; _ledger.append(1)
# Negative end past the front → empty
assert a[:-100] == []; _ledger.append(1)

# String slice with step
s = "abcdefghij"
assert s[::2] == "acegi"; _ledger.append(1)
assert s[::3] == "adgj"; _ledger.append(1)
# String reverse
assert s[::-1] == "jihgfedcba"; _ledger.append(1)
# String start/end slice
assert s[1:5] == "bcde"; _ledger.append(1)
assert s[-3:] == "hij"; _ledger.append(1)
assert s[:3] == "abc"; _ledger.append(1)

# Tuple slice with step — preserves tuple type
t = (0, 1, 2, 3, 4, 5)
assert t[::2] == (0, 2, 4); _ledger.append(1)
assert t[::-1] == (5, 4, 3, 2, 1, 0); _ledger.append(1)
assert t[1:5] == (1, 2, 3, 4); _ledger.append(1)
assert t[:] == (0, 1, 2, 3, 4, 5); _ledger.append(1)
# Type is preserved (not promoted to list)
assert type(t[::2]).__name__ == "tuple"; _ledger.append(1)

# Single-element slice (a degenerate range)
assert a[2:3] == [2]; _ledger.append(1)
assert s[2:3] == "c"; _ledger.append(1)
assert t[2:3] == (2,); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_slice_step {sum(_ledger)} asserts")
