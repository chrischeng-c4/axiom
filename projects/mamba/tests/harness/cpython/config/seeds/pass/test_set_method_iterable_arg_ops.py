# Operational AssertionPass seed for the matching set / frozenset
# method surface where the ARGUMENT IS AN ITERABLE-BUT-NOT-A-SET —
# specifically list / tuple / generator-expression. CPython documents
# that every set method accepting a "set-like" parameter — .union,
# .intersection, .difference, .symmetric_difference, .update,
# .difference_update, .intersection_update,
# .symmetric_difference_update, .issubset, .issuperset, .isdisjoint —
# is defined to accept ANY iterable. The existing pass seeds
# (test_set_algebra_ops, test_set_method_api_ops,
# test_set_mutation_ops) heavily cover the set-vs-set forms but the
# iterable-arg form has only `.update([3,4,5])` covered, leaving the
# union/intersection/difference/symmetric_difference (read-only),
# difference_update / intersection_update /
# symmetric_difference_update (mutating), and issubset / issuperset /
# isdisjoint (predicate) families effectively unverified for the
# list / tuple / generator-expression argument shapes on both runtimes.
#
# Surface (the matching subset between mamba and CPython):
#   • read-only ops on iterable arg:
#       set.union(list/tuple/gen),
#       set.intersection(list/tuple/gen),
#       set.difference(list/tuple/gen),
#       set.symmetric_difference(list/tuple/gen);
#   • mutating ops on iterable arg:
#       set.update(list/tuple), set.difference_update(list/tuple),
#       set.intersection_update(list/tuple),
#       set.symmetric_difference_update(list/tuple);
#   • predicate ops on iterable arg:
#       set.issubset(list/tuple/gen),
#       set.issuperset(list/tuple/gen),
#       set.isdisjoint(list/tuple/gen);
#   • frozenset(list/tuple) constructor + method form on iterable;
#   • set(list/tuple) constructor round-trip;
#   • empty-iterable edge cases for every family.
_ledger: list[int] = []

# set.union(iterable) — list / tuple / generator / empty
assert {1, 2, 3}.union([3, 4, 5]) == {1, 2, 3, 4, 5}; _ledger.append(1)
assert {1, 2, 3}.union([4, 5, 6]) == {1, 2, 3, 4, 5, 6}; _ledger.append(1)
assert {1, 2, 3}.union((4, 5)) == {1, 2, 3, 4, 5}; _ledger.append(1)
assert {1, 2, 3}.union((4,)) == {1, 2, 3, 4}; _ledger.append(1)
assert {1, 2}.union(x for x in [3, 4]) == {1, 2, 3, 4}; _ledger.append(1)
assert {1, 2}.union([]) == {1, 2}; _ledger.append(1)
assert set().union([1, 2, 3]) == {1, 2, 3}; _ledger.append(1)
assert set().union([]) == set(); _ledger.append(1)

# set.intersection(iterable)
assert {1, 2, 3}.intersection([2, 3, 4]) == {2, 3}; _ledger.append(1)
assert {1, 2, 3}.intersection([3]) == {3}; _ledger.append(1)
assert {1, 2, 3}.intersection((2, 4)) == {2}; _ledger.append(1)
assert {1, 2, 3}.intersection((1, 2, 3)) == {1, 2, 3}; _ledger.append(1)
assert {1, 2, 3}.intersection(x for x in [2, 4]) == {2}; _ledger.append(1)
assert {1, 2, 3}.intersection([]) == set(); _ledger.append(1)
assert set().intersection([1, 2, 3]) == set(); _ledger.append(1)

# set.difference(iterable)
assert {1, 2, 3}.difference([2, 3]) == {1}; _ledger.append(1)
assert {1, 2, 3}.difference([1, 2, 3]) == set(); _ledger.append(1)
assert {1, 2, 3}.difference((1,)) == {2, 3}; _ledger.append(1)
assert {1, 2, 3}.difference(x for x in [1, 2]) == {3}; _ledger.append(1)
assert {1, 2, 3}.difference([]) == {1, 2, 3}; _ledger.append(1)
assert set().difference([1, 2, 3]) == set(); _ledger.append(1)

# set.symmetric_difference(iterable)
assert {1, 2, 3}.symmetric_difference([3, 4]) == {1, 2, 4}; _ledger.append(1)
assert {1, 2}.symmetric_difference((2, 3)) == {1, 3}; _ledger.append(1)
assert {1, 2}.symmetric_difference((3,)) == {1, 2, 3}; _ledger.append(1)
assert {1, 2}.symmetric_difference(x for x in [2, 3]) == {1, 3}; _ledger.append(1)
assert {1, 2}.symmetric_difference([]) == {1, 2}; _ledger.append(1)
assert set().symmetric_difference([1, 2]) == {1, 2}; _ledger.append(1)

# set.update(iterable) — mutating
s = {1, 2}
s.update([3, 4])
assert s == {1, 2, 3, 4}; _ledger.append(1)
s = {1, 2}
s.update((3, 4))
assert s == {1, 2, 3, 4}; _ledger.append(1)
s = {1, 2}
s.update([2, 3])
assert s == {1, 2, 3}; _ledger.append(1)
s = {1, 2}
s.update([])
assert s == {1, 2}; _ledger.append(1)

# set.difference_update(iterable) — mutating
s = {1, 2, 3}
s.difference_update([1, 2])
assert s == {3}; _ledger.append(1)
s = {1, 2, 3}
s.difference_update((1,))
assert s == {2, 3}; _ledger.append(1)
s = {1, 2, 3}
s.difference_update([])
assert s == {1, 2, 3}; _ledger.append(1)
s = {1, 2, 3}
s.difference_update([99])
assert s == {1, 2, 3}; _ledger.append(1)

# set.intersection_update(iterable) — mutating
s = {1, 2, 3}
s.intersection_update([2, 3, 4])
assert s == {2, 3}; _ledger.append(1)
s = {1, 2, 3}
s.intersection_update((2,))
assert s == {2}; _ledger.append(1)
s = {1, 2, 3}
s.intersection_update([])
assert s == set(); _ledger.append(1)
s = {1, 2, 3}
s.intersection_update([1, 2, 3])
assert s == {1, 2, 3}; _ledger.append(1)

# set.symmetric_difference_update(iterable) — mutating
s = {1, 2, 3}
s.symmetric_difference_update([3, 4])
assert s == {1, 2, 4}; _ledger.append(1)
s = {1, 2}
s.symmetric_difference_update((2, 3))
assert s == {1, 3}; _ledger.append(1)
s = {1, 2}
s.symmetric_difference_update([])
assert s == {1, 2}; _ledger.append(1)

# set.issubset(iterable)
assert {1, 2}.issubset([1, 2, 3]); _ledger.append(1)
assert {1, 2}.issubset((1, 2, 3)); _ledger.append(1)
assert not {1, 4}.issubset([1, 2, 3]); _ledger.append(1)
assert set().issubset([1, 2, 3]); _ledger.append(1)
assert set().issubset([]); _ledger.append(1)
assert {1, 2}.issubset(x for x in [1, 2, 3]); _ledger.append(1)
assert {1, 2, 3}.issubset([1, 2, 3]); _ledger.append(1)

# set.issuperset(iterable)
assert {1, 2, 3}.issuperset([1, 2]); _ledger.append(1)
assert {1, 2, 3}.issuperset((1, 2)); _ledger.append(1)
assert not {1, 2, 3}.issuperset([1, 4]); _ledger.append(1)
assert {1, 2, 3}.issuperset([]); _ledger.append(1)
assert set().issuperset([]); _ledger.append(1)
assert {1, 2, 3}.issuperset(x for x in [1, 2]); _ledger.append(1)
assert {1, 2, 3}.issuperset([1, 2, 3]); _ledger.append(1)

# set.isdisjoint(iterable)
assert {1, 2}.isdisjoint([3, 4]); _ledger.append(1)
assert {1, 2}.isdisjoint((3, 4)); _ledger.append(1)
assert not {1, 2}.isdisjoint([2, 3]); _ledger.append(1)
assert set().isdisjoint([1, 2, 3]); _ledger.append(1)
assert {1, 2}.isdisjoint([]); _ledger.append(1)
assert {1, 2}.isdisjoint(x for x in [3, 4]); _ledger.append(1)

# frozenset(iterable) constructor + method form
assert frozenset([1, 2, 3]) == frozenset({1, 2, 3}); _ledger.append(1)
assert frozenset((1, 2, 3)) == frozenset({1, 2, 3}); _ledger.append(1)
assert frozenset([1, 2, 2, 3]) == frozenset({1, 2, 3}); _ledger.append(1)
assert frozenset() == frozenset([]); _ledger.append(1)
assert frozenset({1, 2}).union([3, 4]) == frozenset({1, 2, 3, 4}); _ledger.append(1)
assert frozenset({1, 2, 3}).intersection([2, 3, 4]) == frozenset({2, 3}); _ledger.append(1)
assert frozenset({1, 2, 3}).difference([2]) == frozenset({1, 3}); _ledger.append(1)
assert frozenset({1, 2}).symmetric_difference([2, 3]) == frozenset({1, 3}); _ledger.append(1)

# set(iterable) constructor round-trip
assert set([1, 2, 2, 3]) == {1, 2, 3}; _ledger.append(1)
assert set((1, 2, 3, 3)) == {1, 2, 3}; _ledger.append(1)
assert set([]) == set(); _ledger.append(1)
assert set([1]) == {1}; _ledger.append(1)
assert set([1, 1, 1]) == {1}; _ledger.append(1)

# Constructor consumes a generator
assert set(x * 2 for x in [1, 2, 3]) == {2, 4, 6}; _ledger.append(1)
assert frozenset(x for x in [1, 2, 2, 3]) == frozenset({1, 2, 3}); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_set_method_iterable_arg_ops {sum(_ledger)} asserts")
