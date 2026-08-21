# Operational AssertionPass seed for set mutation methods + frozenset
# beyond test_set_ops basics.
# Surface: .pop returning a value and dropping it from the set,
# .clear emptying the set, .update from iterable, in-place
# intersection/difference/symmetric_difference updates, frozenset
# constructor deduplication + hashability.
_ledger: list[int] = []

# .pop removes and returns an arbitrary element; the result count drops
sp = {10, 20, 30}
v = sp.pop()
assert v in {10, 20, 30}; _ledger.append(1)
assert len(sp) == 2; _ledger.append(1)

# .clear empties the set in place
sc = {1, 2, 3}
sc.clear()
assert len(sc) == 0; _ledger.append(1)
assert sc == set(); _ledger.append(1)

# .update consumes an iterable, adding any missing elements
su = {1, 2}
su.update([3, 4, 5])
assert su == {1, 2, 3, 4, 5}; _ledger.append(1)

# .intersection_update keeps only the elements also in the argument
si = {1, 2, 3, 4, 5}
si.intersection_update({2, 3, 4})
assert si == {2, 3, 4}; _ledger.append(1)

# .difference_update removes elements that are in the argument
sd = {1, 2, 3, 4, 5}
sd.difference_update({2, 3})
assert sd == {1, 4, 5}; _ledger.append(1)

# .symmetric_difference_update keeps elements in exactly one of the
# original or the argument
ss = {1, 2, 3}
ss.symmetric_difference_update({2, 3, 4})
assert ss == {1, 4}; _ledger.append(1)

# frozenset deduplicates its input and is hashable
fs = frozenset([1, 2, 3, 2])
assert len(fs) == 3; _ledger.append(1)
# frozenset compares equal to a set with the same elements
assert fs == {1, 2, 3}; _ledger.append(1)
# Hashability: frozenset can be a dict key
d = {fs: "ok"}
assert d[fs] == "ok"; _ledger.append(1)
# frozenset supports the same set algebra as set
assert fs | frozenset([4]) == frozenset([1, 2, 3, 4]); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_set_mutation_ops {sum(_ledger)} asserts")
