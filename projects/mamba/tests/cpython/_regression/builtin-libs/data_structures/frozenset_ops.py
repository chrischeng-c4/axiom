# Frozenset operations: R6 algebra, R7 immutability, R8 len/contains
fs = frozenset([1, 2, 3])
print(len(fs))
print(2 in fs)
print(9 in fs)
# R6: algebra with frozenset operands
a = frozenset([1, 2, 3])
b = frozenset([2, 3, 4])
print(sorted(a.union(b)))
print(sorted(a.intersection(b)))
print(sorted(a.difference(b)))
print(sorted(a.symmetric_difference(b)))
# R7: mutation raises AttributeError
try:
    fs.add(4)
except AttributeError:
    print('caught add')
