# Set in-place update methods: intersection_update, difference_update,
# symmetric_difference_update. CPython mutates in place and returns None.
# Mamba previously raised AttributeError for all three.

# intersection_update: keep only elements also in the argument
s = {1, 2, 3, 4}
r = s.intersection_update({2, 3, 5})
print(r)                              # None
print(sorted(s))                      # [2, 3]

# difference_update: drop every element also in the argument
s = {1, 2, 3, 4}
r = s.difference_update({2, 4})
print(r)                              # None
print(sorted(s))                      # [1, 3]

# symmetric_difference_update: keep elements in exactly one set
s = {1, 2, 3}
r = s.symmetric_difference_update({2, 3, 4})
print(r)                              # None
print(sorted(s))                      # [1, 4]

# Disjoint argument leaves intersection_update empty
s = {1, 2, 3}
s.intersection_update({99})
print(sorted(s))                      # []

# Empty argument leaves difference_update unchanged
s = {1, 2, 3}
s.difference_update(set())
print(sorted(s))                      # [1, 2, 3]

# Identity (sym-diff with self → empty)
s = {1, 2, 3}
s.symmetric_difference_update({1, 2, 3})
print(sorted(s))                      # []

# Accepts list / tuple iterables (matches CPython's any-iterable contract for
# difference_update / intersection_update — symmetric_difference_update is
# stricter in CPython but we accept the same iterables for parity with `update`).
s = {1, 2, 3, 4}
s.difference_update([2, 3])
print(sorted(s))                      # [1, 4]

s = {1, 2, 3, 4}
s.intersection_update((2, 3))
print(sorted(s))                      # [2, 3]

# frozenset rejects all three
try:
    frozenset({1, 2}).difference_update({1})
except AttributeError as e:
    print("frozenset diff_update raised AttributeError:", "difference_update" in str(e))
