# Operational AssertionPass seed for the slice-notation surface.
# Surface: `seq[i:j]` returns the half-open `[i, j)` slice; either
# bound may be omitted (`seq[:j]` / `seq[i:]` / `seq[:]` for full
# copy); negative indices are interpreted from the end (`seq[-3:]`
# is the last three); `seq[i:j:k]` adds a stride (k=2 returns every
# other element, k=-1 reverses, k=-2 reverses with stride);
# slicing past the end / empty/empty / start>stop / 0:0 all
# produce an empty container of the matching kind; works
# uniformly over list, str, tuple, bytes; LHS slice assignment
# replaces the slice with the RHS iterable (same length, grow,
# shrink to empty, full-replace via `m[:]`); `m[0:0] = [...]`
# inserts at the front. Companion to lang_subscript (single index).
_ledger: list[int] = []

# Half-open list slice with explicit + omitted bounds
L = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
assert L[2:5] == [2, 3, 4]; _ledger.append(1)
assert L[:3] == [0, 1, 2]; _ledger.append(1)
assert L[7:] == [7, 8, 9]; _ledger.append(1)
assert L[:] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)

# Negative indices index from the end
assert L[-3:] == [7, 8, 9]; _ledger.append(1)
assert L[:-3] == [0, 1, 2, 3, 4, 5, 6]; _ledger.append(1)
assert L[-5:-2] == [5, 6, 7]; _ledger.append(1)

# Stride — positive, every-other, every-third
assert L[::2] == [0, 2, 4, 6, 8]; _ledger.append(1)
assert L[1::2] == [1, 3, 5, 7, 9]; _ledger.append(1)
assert L[::3] == [0, 3, 6, 9]; _ledger.append(1)

# Negative stride — reverse and stride-reverse
assert L[::-1] == [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]; _ledger.append(1)
assert L[5:2:-1] == [5, 4, 3]; _ledger.append(1)
assert L[::-2] == [9, 7, 5, 3, 1]; _ledger.append(1)

# Empty results — past end / empty range / inverted / zero-width
assert L[10:] == []; _ledger.append(1)
assert L[5:5] == []; _ledger.append(1)
assert L[5:2] == []; _ledger.append(1)
assert L[0:0] == []; _ledger.append(1)

# Strings — slice produces str
s = "hello world"
assert s[0:5] == "hello"; _ledger.append(1)
assert s[6:] == "world"; _ledger.append(1)
assert s[:5] == "hello"; _ledger.append(1)
assert s[-5:] == "world"; _ledger.append(1)
assert s[::-1] == "dlrow olleh"; _ledger.append(1)
assert s[::2] == "hlowrd"; _ledger.append(1)
assert s[6:11] == "world"; _ledger.append(1)
assert s[1:9:2] == "el o"; _ledger.append(1)

# Tuples — slice produces tuple
t = (1, 2, 3, 4, 5)
assert t[1:4] == (2, 3, 4); _ledger.append(1)
assert t[:3] == (1, 2, 3); _ledger.append(1)
assert t[::-1] == (5, 4, 3, 2, 1); _ledger.append(1)
assert t[::2] == (1, 3, 5); _ledger.append(1)

# Bytes — slice produces bytes
bb = b"hello"
assert bb[0:3] == b"hel"; _ledger.append(1)
assert bb[::-1] == b"olleh"; _ledger.append(1)
assert bb[1:] == b"ello"; _ledger.append(1)

# range(n) → list — stride composition
r = list(range(10))
assert r[2:8:2] == [2, 4, 6]; _ledger.append(1)

# LHS slice assignment — same length
m = [0, 1, 2, 3, 4]
m[1:3] = [10, 20]
assert m == [0, 10, 20, 3, 4]; _ledger.append(1)

# LHS slice assignment — grow (RHS longer than slice)
m2 = [0, 1, 2, 3, 4]
m2[1:3] = [10, 20, 30]
assert m2 == [0, 10, 20, 30, 3, 4]; _ledger.append(1)

# LHS slice assignment — shrink to empty
m3 = [0, 1, 2, 3, 4]
m3[1:3] = []
assert m3 == [0, 3, 4]; _ledger.append(1)

# Full replacement via m[:] = ...
m4 = [0, 1, 2, 3, 4]
m4[:] = [100]
assert m4 == [100]; _ledger.append(1)

# Front-insert via zero-width LHS slice
ll = [1, 2, 3]
ll[0:0] = [-1, 0]
assert ll == [-1, 0, 1, 2, 3]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_slice_notation {sum(_ledger)} asserts")
