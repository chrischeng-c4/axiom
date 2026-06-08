# RUN: parse
# CPython 3.12 test_set: set operations

# Construction
s = set()
s = {1, 2, 3}
s = set([1, 2, 3])
s = set("hello")

# Modification
s = {1, 2, 3}
s.add(4)
s.remove(1)
s.discard(10)
v = s.pop()
s.clear()
s.update([4, 5])
s.add(6)

# Set operations
a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
union = a | b
union = a.union(b)
inter = a & b
inter = a.intersection(b)
diff = a - b
diff = a.difference(b)
sym_diff = a ^ b
sym_diff = a.symmetric_difference(b)

# Subset / superset
is_sub = {1, 2}.issubset({1, 2, 3})
is_super = {1, 2, 3}.issuperset({1, 2})
is_disjoint = {1, 2}.isdisjoint({3, 4})

# In-place operations
a |= b
a &= b
a -= b
a ^= b

# Membership
n = 1 in {1, 2, 3}
n = 5 not in {1, 2, 3}

# Set comprehension
squares = {x**2 for x in range(10)}
evens = {x for x in range(10) if x % 2 == 0}

# len / copy
n = len(s)
s2 = s.copy()

# frozenset
fs = frozenset({1, 2, 3})
fs2 = frozenset([1, 2, 3])
