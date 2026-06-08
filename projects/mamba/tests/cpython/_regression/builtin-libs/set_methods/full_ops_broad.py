# set operations broad

# set creation
s = {1, 2, 3}
print(sorted(s))

s2 = set()
print(len(s2))

s3 = set([1, 2, 3, 2, 1])
print(sorted(s3))
print(len(s3))

s4 = set((1, 2, 3))
print(sorted(s4))

s5 = set("abc")
print(sorted(s5))

# add
s = {1, 2}
s.add(3)
s.add(1)  # no-op
print(sorted(s))
print(len(s))

# remove
s = {1, 2, 3}
s.remove(2)
print(sorted(s))

# discard (no error)
s = {1, 2, 3}
s.discard(99)
s.discard(2)
print(sorted(s))

# pop
s = {1}
p = s.pop()
print(p)
print(len(s))

# clear
s = {1, 2, 3}
s.clear()
print(len(s))

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
print(sorted(a.difference(b)))

# symmetric difference
print(sorted(a ^ b))
print(sorted(a.symmetric_difference(b)))

# subset / superset
s1 = {1, 2}
s2 = {1, 2, 3}
print(s1 <= s2)
print(s2 >= s1)
print(s1 < s2)
print(s1 > s2)

# membership
s = {1, 2, 3}
print(1 in s)
print(99 in s)
print(1 not in s)
print(99 not in s)

# iterate set
s = {1, 2, 3}
total = 0
for x in s:
    total += x
print(total)

# set comprehension
sc = {x * 2 for x in range(5)}
print(sorted(sc))

sc2 = {x for x in range(10) if x % 2 == 0}
print(sorted(sc2))

# update (union in-place)
a = {1, 2, 3}
a.update({4, 5})
print(sorted(a))

# disjoint
print({1, 2}.isdisjoint({3, 4}))
print({1, 2}.isdisjoint({2, 3}))

# empty set arithmetic
e = set()
print(sorted(e | {1, 2}))
print(sorted({1, 2} | e))
print(len(e & {1, 2}))
print(sorted({1, 2} - e))

# copy
a = {1, 2, 3}
b = a.copy()
b.add(4)
print(sorted(a))
print(sorted(b))

# frozenset
fs = frozenset([1, 2, 3])
print(sorted(fs))
print(len(fs))
print(1 in fs)
