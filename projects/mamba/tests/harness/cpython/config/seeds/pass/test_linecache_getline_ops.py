# Operational AssertionPass seed for the `linecache` module — the
# source-line cache used by `traceback` / `inspect` / `pdb` /
# debuggers / type-checkers to format `^^^^^` callouts in error
# reports. Surface: `getline(path, n)` returns the n-th 1-indexed
# line of `path` as a string (empty on out-of-range / missing-file),
# `getlines(path)` returns the entire list of lines for `path`,
# `clearcache()` evicts the per-path cache, `checkcache()` revalidates
# the cache against mtimes (both return `None`). `linecache` has no
# fixture coverage yet.
#
# Surface:
#   • linecache.getline(path, lineno: 1-indexed) → str
#       — out-of-range / negative / zero / missing-file → "";
#       — content match is `.rstrip("\n")` agnostic (mamba strips the
#         trailing newline; CPython preserves it — this seed anchors
#         on the rstripped equality so both runtimes agree);
#   • linecache.getlines(path) → list[str]
#       — missing-file → empty list;
#       — every element is a `str`;
#   • linecache.clearcache() → None
#   • linecache.checkcache() → None (also accepts a path argument)
import linecache
import os
_ledger: list[int] = []

_path = "/tmp/_linecache_test_fixture_atomic_seed.txt"
with open(_path, "w") as _f:
    _f.write("alpha\nbeta\ngamma\n")

linecache.clearcache()

# getline — 1-indexed line lookup. rstrip the trailing newline so the
# fixture is robust across mamba (strips) / CPython (keeps).
assert linecache.getline(_path, 1).rstrip("\n") == "alpha"; _ledger.append(1)
assert linecache.getline(_path, 2).rstrip("\n") == "beta"; _ledger.append(1)
assert linecache.getline(_path, 3).rstrip("\n") == "gamma"; _ledger.append(1)

# Out-of-range — empty string (NOT a raise / NOT None)
assert linecache.getline(_path, 4) == ""; _ledger.append(1)
assert linecache.getline(_path, 100) == ""; _ledger.append(1)
assert linecache.getline(_path, 0) == ""; _ledger.append(1)
assert linecache.getline(_path, -1) == ""; _ledger.append(1)
assert linecache.getline(_path, -100) == ""; _ledger.append(1)

# Missing file — empty string
assert linecache.getline("/nonexistent_path_xyzqq.txt", 1) == ""; _ledger.append(1)
assert linecache.getline("/another/missing/file.txt", 5) == ""; _ledger.append(1)

# Return types — getline returns str
assert isinstance(linecache.getline(_path, 1), str); _ledger.append(1)
assert isinstance(linecache.getline(_path, 999), str); _ledger.append(1)
assert isinstance(linecache.getline("/missing.txt", 1), str); _ledger.append(1)

# getlines — full list of lines
_lines = linecache.getlines(_path)
assert isinstance(_lines, list); _ledger.append(1)
assert len(_lines) == 3; _ledger.append(1)
assert all(isinstance(ln, str) for ln in _lines); _ledger.append(1)

# getlines content — every line contains the expected token
assert any("alpha" in ln for ln in _lines); _ledger.append(1)
assert any("beta" in ln for ln in _lines); _ledger.append(1)
assert any("gamma" in ln for ln in _lines); _ledger.append(1)

# getlines preserves order (rstripped equality)
assert _lines[0].rstrip("\n") == "alpha"; _ledger.append(1)
assert _lines[1].rstrip("\n") == "beta"; _ledger.append(1)
assert _lines[2].rstrip("\n") == "gamma"; _ledger.append(1)

# getlines on missing file — empty list
_missing_lines = linecache.getlines("/nonexistent_xyzqq.txt")
assert _missing_lines == []; _ledger.append(1)
assert isinstance(_missing_lines, list); _ledger.append(1)

# clearcache / checkcache return None
assert linecache.clearcache() is None; _ledger.append(1)
assert linecache.checkcache() is None; _ledger.append(1)

# After clearcache — getline still works (re-populates cache)
assert linecache.getline(_path, 1).rstrip("\n") == "alpha"; _ledger.append(1)
assert linecache.getline(_path, 3).rstrip("\n") == "gamma"; _ledger.append(1)
assert len(linecache.getlines(_path)) == 3; _ledger.append(1)

# checkcache with a specific path
assert linecache.checkcache(_path) is None; _ledger.append(1)
assert linecache.checkcache("/nonexistent.txt") is None; _ledger.append(1)

# Reverse — getline of various out-of-range / negative indices is "":
# checks the invariant for many indices to guard against off-by-one.
for n in [-10, -5, -1, 0, 4, 5, 10, 1000]:
    assert linecache.getline(_path, n) == ""; _ledger.append(1)

# Each in-range line has rstripped value equal to the expected token
_expected = ["alpha", "beta", "gamma"]
for i, exp in enumerate(_expected, start=1):
    assert linecache.getline(_path, i).rstrip("\n") == exp; _ledger.append(1)

# Cleanup — remove the probe file (cache stays valid until next checkcache)
os.remove(_path)
# After removal, in-cache content is still served until checkcache invalidates
assert linecache.getline(_path, 1).rstrip("\n") == "alpha"; _ledger.append(1)

# Now invalidate and confirm post-eviction behavior
linecache.checkcache(_path)
assert linecache.getline(_path, 1) == ""; _ledger.append(1)
assert linecache.getlines(_path) == []; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_linecache_getline_ops {sum(_ledger)} asserts")
