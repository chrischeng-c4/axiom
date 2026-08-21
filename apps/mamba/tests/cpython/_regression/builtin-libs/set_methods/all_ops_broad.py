# set broad ops

a = {1, 2, 3, 4}
b = {3, 4, 5, 6}

# length / contains
print(len(a))
print(1 in a)
print(5 in a)
print(5 not in a)

# union / intersection / difference / symmetric_difference
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

# subset / superset
print({1, 2}.issubset(a))
print(a.issuperset({1, 2}))
print({1, 5}.isdisjoint({2, 6}))
print({1, 5}.isdisjoint({1, 6}))
print({1, 2, 3} == {3, 2, 1})
print({1, 2} != {1, 2, 3})

# add / discard
s = {1, 2, 3}
s.add(4)
print(sorted(s))
s.discard(2)
print(sorted(s))
s.discard(99)
print(sorted(s))

# copy
s2 = a.copy()
s2.add(99)
print(sorted(a))
print(sorted(s2))

# frozenset
fs = frozenset([1, 2, 3])
print(sorted(fs))
print(1 in fs)
print(len(fs))

# set from iterable
print(sorted(set([1, 2, 2, 3, 3, 3])))
print(sorted(set(range(5))))

# set comp
print(sorted({x * 2 for x in range(5)}))
print(sorted({x % 3 for x in range(10)}))
