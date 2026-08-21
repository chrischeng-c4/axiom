# Operational AssertionPass seed for sorted/min/max with key=.
# Surface: sorted by len, sorted by abs, min/max by key, reverse=True,
# stable sort preserving input order on key ties.
# Companion to stub/test_sort.py — vendored unittest seed.
_ledger: list[int] = []
assert sorted(["bb", "a", "ccc"], key=len) == ["a", "bb", "ccc"]; _ledger.append(1)
assert sorted([3, -1, -4, 1], key=abs) == [-1, 1, 3, -4]; _ledger.append(1)
desc = sorted([3, -1, -4, 1], key=abs, reverse=True)
assert desc[0] == -4 and desc[1] == 3; _ledger.append(1)
assert sorted(desc[2:]) == [-1, 1]; _ledger.append(1)
assert min(["aa", "b", "ccc"], key=len) == "b"; _ledger.append(1)
assert max([-3, -1, 4], key=abs) == 4; _ledger.append(1)
assert sorted([1, 2, 3], reverse=True) == [3, 2, 1]; _ledger.append(1)
# Stable sort: equal keys preserve relative order
pairs = [(1, "a"), (2, "b"), (1, "c"), (2, "d")]
assert sorted(pairs, key=lambda p: p[0]) == [(1, "a"), (1, "c"), (2, "b"), (2, "d")]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_sorted_key_ops {sum(_ledger)} asserts")
