# Set operations
s = {1, 2, 3}
s.add(4)
print(sorted(s))
s.discard(2)
print(sorted(s))
s.remove(3)
print(sorted(s))
# Union, intersection, difference
a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
print(sorted(a | b))
print(sorted(a & b))
print(sorted(a - b))
print(sorted(a ^ b))
# Method forms
print(sorted(a.union(b)))
print(sorted(a.intersection(b)))
print(sorted(a.difference(b)))
print(sorted(a.symmetric_difference(b)))
# Subset/superset
print({1, 2}.issubset({1, 2, 3}))
print({1, 2, 3}.issuperset({1, 2}))
# Set comprehension
print(sorted({x ** 2 for x in range(5)}))
# frozenset
fs = frozenset([3, 1, 2])
print(sorted(fs))
print(1 in fs)
# clear
s2 = {1, 2, 3}
s2.clear()
print(len(s2))
