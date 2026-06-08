# Operational AssertionPass seed for builtin `frozenset`.
# Surface: constructor from list, len, in/not in, iteration via
# sorted membership round-trip.
# Companion to stub/test_frozenset.py — vendored unittest seed.
_ledger: list[int] = []
fs = frozenset([1, 2, 3])
assert len(fs) == 3; _ledger.append(1)
assert 2 in fs; _ledger.append(1)
assert 5 not in fs; _ledger.append(1)
assert sorted(fs) == [1, 2, 3]; _ledger.append(1)
empty = frozenset()
assert len(empty) == 0; _ledger.append(1)
assert 0 not in empty; _ledger.append(1)
dup = frozenset([1, 1, 2, 2, 3])
assert len(dup) == 3; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_frozenset_ops {sum(_ledger)} asserts")
