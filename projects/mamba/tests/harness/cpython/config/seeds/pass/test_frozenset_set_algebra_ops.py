# Operational AssertionPass seed for the frozenset set-algebra
# surface. Surface: `frozenset(iterable)` constructs an immutable
# set from list/tuple/str/set; `len(fs)`, `x in fs`, `x not in fs`
# work the same as on `set`; equality is set-equality (order-
# independent); the empty-frozenset is canonical (`frozenset() ==
# frozenset([])`); `issubset`/`issuperset`/`isdisjoint` predicates
# behave like their `set` counterparts; the four set-algebra
# operators `|` union, `&` intersection, `-` difference, `^`
# symmetric-difference each produce a new frozenset with the
# matching contents, and have method aliases `union` /
# `intersection` / `difference` / `symmetric_difference`;
# frozensets are hashable (usable as dict keys), and the empty
# frozenset is a subset of every frozenset including itself;
# `fs.copy()` returns a frozenset equal to fs.
_ledger: list[int] = []

# Construction from list / tuple / str / set; len, membership
fs = frozenset([1, 2, 3])
assert len(fs) == 3; _ledger.append(1)
assert 1 in fs; _ledger.append(1)
assert 4 not in fs; _ledger.append(1)

# Equality is order-independent set-equality
assert fs == frozenset([3, 2, 1]); _ledger.append(1)
assert fs == frozenset({3, 1, 2}); _ledger.append(1)
assert frozenset([1, 2, 3]) == frozenset([3, 2, 1]); _ledger.append(1)

# Empty-frozenset is canonical
assert frozenset() == frozenset([]); _ledger.append(1)
assert len(frozenset()) == 0; _ledger.append(1)

# Construct from str — frozenset of distinct chars
fs2 = frozenset("hello")
assert "h" in fs2; _ledger.append(1)
assert "e" in fs2; _ledger.append(1)
assert "x" not in fs2; _ledger.append(1)
assert len(fs2) == 4; _ledger.append(1)

# Construct from tuple — same content as from list
assert frozenset((1, 2, 3)) == frozenset([1, 2, 3]); _ledger.append(1)

# Subset / superset predicates
assert frozenset([1, 2]).issubset(frozenset([1, 2, 3])); _ledger.append(1)
assert frozenset([1, 2, 3]).issuperset(frozenset([1, 2])); _ledger.append(1)
assert not frozenset([1, 5]).issubset(frozenset([1, 2, 3])); _ledger.append(1)

# Set-algebra operators on prepared sets
a = frozenset([1, 2, 3])
b = frozenset([3, 4, 5])
assert a | b == frozenset([1, 2, 3, 4, 5]); _ledger.append(1)
assert a & b == frozenset([3]); _ledger.append(1)
assert a - b == frozenset([1, 2]); _ledger.append(1)
assert a ^ b == frozenset([1, 2, 4, 5]); _ledger.append(1)

# Method aliases match the operator forms
assert a.union(b) == frozenset([1, 2, 3, 4, 5]); _ledger.append(1)
assert a.intersection(b) == frozenset([3]); _ledger.append(1)
assert a.difference(b) == frozenset([1, 2]); _ledger.append(1)
assert a.symmetric_difference(b) == frozenset([1, 2, 4, 5]); _ledger.append(1)

# Hashable — usable as dict keys
d = {a: "first", b: "second"}
assert d[a] == "first"; _ledger.append(1)
assert d[b] == "second"; _ledger.append(1)

# Disjoint predicate
assert a.isdisjoint(frozenset([10, 20])); _ledger.append(1)
assert not a.isdisjoint(b); _ledger.append(1)

# Empty-set subset relations
assert frozenset().issubset(a); _ledger.append(1)
assert frozenset().issubset(frozenset()); _ledger.append(1)

# Reflexive equality and (sub|super)set
assert a == a; _ledger.append(1)
assert a <= a; _ledger.append(1)
assert a >= a; _ledger.append(1)

# copy returns an equal frozenset
assert a.copy() == a; _ledger.append(1)
assert isinstance(a.copy(), frozenset); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_frozenset_set_algebra_ops {sum(_ledger)} asserts")
