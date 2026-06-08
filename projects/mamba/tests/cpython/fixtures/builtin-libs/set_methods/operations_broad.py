# set operations broad

# basic set
s = {1, 2, 3, 4, 5}
print(sorted(s))
print(len(s))

# add
s2 = {1, 2, 3}
s2.add(4)
s2.add(5)
s2.add(1)  # duplicate
print(sorted(s2))
print(len(s2))

# remove
s3 = {1, 2, 3, 4, 5}
s3.remove(3)
print(sorted(s3))
print(len(s3))

# discard (silent)
s4 = {1, 2, 3}
s4.discard(2)
s4.discard(99)  # not there, ok
print(sorted(s4))

# union
a = {1, 2, 3}
b = {3, 4, 5}
print(sorted(a | b))
print(sorted(a.union(b)))

# intersection
print(sorted(a & b))
print(sorted(a.intersection(b)))

# difference
print(sorted(a - b))
print(sorted(b - a))
print(sorted(a.difference(b)))

# symmetric difference
print(sorted(a ^ b))
print(sorted(a.symmetric_difference(b)))

# subset / superset
x = {1, 2}
y = {1, 2, 3, 4}
print(x <= y)
print(y >= x)
print(x.issubset(y))
print(y.issuperset(x))
print(x < y)
print(y > x)

# equality
print({1, 2, 3} == {3, 2, 1})
print({1, 2} == {1, 2, 3})

# in
print(2 in {1, 2, 3})
print(99 in {1, 2, 3})

# set() constructor
print(sorted(set([1, 2, 2, 3, 3, 3])))
print(len(set([1, 1, 2, 2, 3, 3])))

# set comprehension
print(sorted({x * x for x in range(5)}))
print(sorted({x % 3 for x in range(10)}))

# iterate
tmp = {1, 2, 3}
vals = []
for x in tmp:
    vals.append(x)
print(sorted(vals))

# copy
s5 = {1, 2, 3}
s6 = s5.copy()
s6.add(4)
print(sorted(s5))
print(sorted(s6))

# frozenset
fs = frozenset([1, 2, 3])
print(sorted(fs))
print(len(fs))
print(1 in fs)
