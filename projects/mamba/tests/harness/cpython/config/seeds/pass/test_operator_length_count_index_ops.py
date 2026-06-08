# Operational AssertionPass seed for `operator.length_hint`,
# `operator.countOf`, and `operator.indexOf`. Surface:
# `length_hint(obj)` returns the exact `len()` for sized containers
# (list, tuple, str, set, dict) and accepts an optional `default`
# argument that is ignored when the container is sized — including
# when the container is empty. `countOf(seq, value)` returns the
# integer count of equal elements scanned linearly through `seq`,
# and `indexOf(seq, value)` returns the integer index of the first
# equal element. The string-substring overload of `countOf` and the
# string-substring path of `contains` are NOT asserted here — those
# subsurfaces are tracked separately.
import operator
_ledger: list[int] = []

# length_hint on sized containers ignores the default
assert operator.length_hint([1, 2, 3]) == 3; _ledger.append(1)
assert operator.length_hint([1, 2, 3], 999) == 3; _ledger.append(1)
assert operator.length_hint([], 99) == 0; _ledger.append(1)
assert operator.length_hint([1], 50) == 1; _ledger.append(1)
assert operator.length_hint("abc") == 3; _ledger.append(1)
assert operator.length_hint("") == 0; _ledger.append(1)
assert operator.length_hint((1, 2, 3, 4)) == 4; _ledger.append(1)
assert operator.length_hint(()) == 0; _ledger.append(1)
assert operator.length_hint({1, 2, 3}) == 3; _ledger.append(1)
assert operator.length_hint({"a": 1, "b": 2}) == 2; _ledger.append(1)
assert operator.length_hint({}) == 0; _ledger.append(1)

# countOf — linear count of equal elements
assert operator.countOf([1, 2, 2, 3, 2], 2) == 3; _ledger.append(1)
assert operator.countOf([1, 2, 3], 99) == 0; _ledger.append(1)
assert operator.countOf([], 1) == 0; _ledger.append(1)
assert operator.countOf((5, 5, 5), 5) == 3; _ledger.append(1)

# indexOf — first matching index
assert operator.indexOf([10, 20, 30], 20) == 1; _ledger.append(1)
assert operator.indexOf([10, 20, 30], 10) == 0; _ledger.append(1)
assert operator.indexOf([10, 20, 30], 30) == 2; _ledger.append(1)
assert operator.indexOf((1, 2, 3, 2), 2) == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_operator_length_count_index_ops {sum(_ledger)} asserts")
