# set deep broad

# construction
s1 = {1, 2, 3}
print(sorted(s1))

# empty set
s_empty = set()
print(len(s_empty))

# set from list (dedupe)
s2 = set([1, 2, 3, 2, 1, 4])
print(sorted(s2))

# add / discard / remove
s3 = {1, 2, 3}
s3.add(4)
print(sorted(s3))
s3.discard(2)
print(sorted(s3))
s3.discard(999)  # no-op
print(sorted(s3))
s3.remove(1)
print(sorted(s3))

# union, intersection, difference, sym-diff
a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
print(sorted(a | b))
print(sorted(a & b))
print(sorted(a - b))
print(sorted(b - a))
print(sorted(a ^ b))

# method forms
print(sorted(a.union(b)))
print(sorted(a.intersection(b)))
print(sorted(a.difference(b)))
print(sorted(a.symmetric_difference(b)))

# issubset / issuperset / isdisjoint
small = {1, 2}
big = {1, 2, 3, 4}
print(small.issubset(big))
print(big.issuperset(small))
print({1, 2}.isdisjoint({3, 4}))
print({1, 2}.isdisjoint({2, 3}))

# update
s_u = {1, 2}
s_u.update({3, 4})
print(sorted(s_u))

# copy
s_orig = {1, 2, 3}
s_copy = s_orig.copy()
s_copy.add(99)
print(sorted(s_orig))
print(sorted(s_copy))

# len, in
print(len({1, 2, 3}))
print(len(set()))
print(3 in {1, 2, 3})
print(99 in {1, 2, 3})

# frozenset
fs = frozenset([1, 2, 3])
print(sorted(fs))
print(len(fs))

# set comp
print(sorted({x * x for x in range(5)}))
print(sorted({x for x in range(20) if x % 3 == 0}))
