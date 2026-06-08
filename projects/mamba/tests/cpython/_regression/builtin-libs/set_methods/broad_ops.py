a = {1, 2, 3, 4, 5}
b = {3, 4, 5, 6, 7}

# union / intersection / difference / symmetric_difference
print(sorted(a | b))
print(sorted(a.union(b)))
print(sorted(a & b))
print(sorted(a.intersection(b)))
print(sorted(a - b))
print(sorted(a.difference(b)))
print(sorted(a ^ b))
print(sorted(a.symmetric_difference(b)))

# subset / superset / disjoint
print({1, 2}.issubset({1, 2, 3}))
print({1, 2, 3}.issuperset({1, 2}))
print({1, 2}.isdisjoint({3, 4}))
print({1, 2}.isdisjoint({2, 3}))

# set comprehension
print(sorted({x * x for x in range(5)}))
print(sorted({x for x in range(10) if x % 2 == 0}))

# add / remove / discard
s = {1, 2, 3}
s.add(4)
print(sorted(s))
s.discard(2)
print(sorted(s))
s.discard(99)
print(sorted(s))
s.remove(3)
print(sorted(s))

# clear
s = {1, 2, 3}
s.clear()
print(len(s))

# copy
s = {1, 2, 3}
c = s.copy()
print(sorted(c))

# update
s = {1, 2}
s.update([3, 4, 5])
print(sorted(s))

# len / in
print(len({1, 2, 3, 4}))
print(3 in {1, 2, 3})
print(99 in {1, 2, 3})

# frozenset basics
fs = frozenset([1, 2, 3])
print(sorted(fs))
print(1 in fs)
