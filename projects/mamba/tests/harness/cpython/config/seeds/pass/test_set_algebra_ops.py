# Operational AssertionPass seed for set-algebra surfaces beyond the
# basic test_set_ops / test_set_method_api_ops / test_set_mutation_ops
# trio. Surface: the four set-algebra operators (| union, & intersection,
# - difference, ^ symmetric-difference) and their method equivalents
# (.union/.intersection/.difference/.symmetric_difference); in-place
# variants (|=, &=, -=, ^=) and their method equivalents (.update,
# .intersection_update, .difference_update, .symmetric_difference_update);
# subset / superset / proper-subset / proper-superset comparisons via
# <=, <, >=, > and the .issubset / .issuperset methods; .isdisjoint;
# frozenset construction, equality, membership, length, and union with
# a regular set; set comprehension; add / discard (silent on miss) /
# remove (must-exist) / pop / copy.
_ledger: list[int] = []

# Set algebra via operators
a = {1, 2, 3}
b = {2, 3, 4}
assert a | b == {1, 2, 3, 4}; _ledger.append(1)
assert a & b == {2, 3}; _ledger.append(1)
assert a - b == {1}; _ledger.append(1)
assert a ^ b == {1, 4}; _ledger.append(1)

# Same algebra via method names
assert a.union(b) == {1, 2, 3, 4}; _ledger.append(1)
assert a.intersection(b) == {2, 3}; _ledger.append(1)
assert a.difference(b) == {1}; _ledger.append(1)
assert a.symmetric_difference(b) == {1, 4}; _ledger.append(1)

# In-place operator variants
c = {1, 2, 3}
c |= {4, 5}
assert c == {1, 2, 3, 4, 5}; _ledger.append(1)
d = {1, 2, 3, 4}
d &= {2, 4}
assert d == {2, 4}; _ledger.append(1)
e = {1, 2, 3}
e -= {2}
assert e == {1, 3}; _ledger.append(1)
f = {1, 2, 3}
f ^= {2, 4}
assert f == {1, 3, 4}; _ledger.append(1)

# In-place via method names
g = {1, 2}
g.update({3, 4})
assert g == {1, 2, 3, 4}; _ledger.append(1)
h = {1, 2, 3}
h.intersection_update({2, 3, 4})
assert h == {2, 3}; _ledger.append(1)
i = {1, 2, 3}
i.difference_update({2})
assert i == {1, 3}; _ledger.append(1)
j = {1, 2, 3}
j.symmetric_difference_update({2, 4})
assert j == {1, 3, 4}; _ledger.append(1)

# Subset / superset comparisons via operators
assert {1, 2} <= {1, 2, 3}; _ledger.append(1)
assert {1, 2, 3} >= {1, 2}; _ledger.append(1)
assert {1, 2} <= {1, 2}; _ledger.append(1)
# Proper subset / superset require strict inequality (no element overlap)
assert {1, 2} < {1, 2, 3}; _ledger.append(1)
assert {1, 2, 3} > {1, 2}; _ledger.append(1)
# An equal set is NOT a proper subset/superset
assert not ({1, 2} < {1, 2}); _ledger.append(1)
assert not ({1, 2} > {1, 2}); _ledger.append(1)

# Same via .issubset / .issuperset method names
assert {1, 2}.issubset({1, 2, 3}); _ledger.append(1)
assert {1, 2, 3}.issuperset({1, 2}); _ledger.append(1)

# isdisjoint — true when the two sets share no element
assert {1, 2}.isdisjoint({3, 4}); _ledger.append(1)
assert not {1, 2}.isdisjoint({2, 3}); _ledger.append(1)
# Empty set is disjoint with anything
assert set().isdisjoint({1, 2, 3}); _ledger.append(1)

# frozenset — immutable set with the same surface
fs = frozenset([1, 2, 3])
assert fs == frozenset([3, 2, 1]); _ledger.append(1)
assert 2 in fs; _ledger.append(1)
assert len(fs) == 3; _ledger.append(1)
# Mixed union — frozenset | set produces a hashable-or-not result that
# compares equal to the algebraic union
assert fs | {4} == {1, 2, 3, 4}; _ledger.append(1)

# Set comprehension
assert {x * 2 for x in [1, 2, 3]} == {2, 4, 6}; _ledger.append(1)
assert {x for x in [1, 1, 2, 2, 3]} == {1, 2, 3}; _ledger.append(1)

# Add / discard / remove — mutation API
k = {1, 2}
k.add(3)
assert k == {1, 2, 3}; _ledger.append(1)
# discard never raises on miss
k.discard(2)
assert k == {1, 3}; _ledger.append(1)
k.discard(99)
assert k == {1, 3}; _ledger.append(1)
# remove must be present (we know 1 is here)
k.remove(1)
assert k == {3}; _ledger.append(1)

# pop returns some element (single-element set is deterministic)
m = {42}
v = m.pop()
assert v == 42; _ledger.append(1)
assert m == set(); _ledger.append(1)

# copy — top-level independent
n = {1, 2, 3}
o = n.copy()
o.add(4)
assert n == {1, 2, 3}; _ledger.append(1)
assert o == {1, 2, 3, 4}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_set_algebra_ops {sum(_ledger)} asserts")
