# Operational AssertionPass seed for the set named-method API and
# ordering predicates.
# Surface: union, intersection, difference, symmetric_difference,
# plus the <, <=, >, >= operators (strict / non-strict subset and
# superset). Complements test_set_ops.py which covers only the
# infix operators (|, &, -, ^).
_ledger: list[int] = []
a = {1, 2, 3}
b = {2, 3, 4}
# Named methods mirror the infix operator results
assert a.union(b) == {1, 2, 3, 4}; _ledger.append(1)
assert a.intersection(b) == {2, 3}; _ledger.append(1)
assert a.difference(b) == {1}; _ledger.append(1)
assert a.symmetric_difference(b) == {1, 4}; _ledger.append(1)
# Subset predicates: < strict, <= non-strict, > strict, >= non-strict
assert {1, 2} < a; _ledger.append(1)
assert a <= {1, 2, 3}; _ledger.append(1)
assert a >= {1, 2}; _ledger.append(1)
assert a > {1, 2}; _ledger.append(1)
# a < a should be False (strict)
assert not (a < a); _ledger.append(1)
# a <= a should be True (non-strict)
assert a <= a; _ledger.append(1)
# Empty set is a subset of every set
empty: set[int] = set()
assert empty <= a; _ledger.append(1)
assert empty < a; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_set_method_api_ops {sum(_ledger)} asserts")
