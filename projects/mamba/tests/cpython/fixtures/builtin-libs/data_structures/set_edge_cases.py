# Set edge cases: frozenset hash, comprehension with filter, clear
# frozenset hash consistency
fs = frozenset([1, 2])
print(hash(fs) == hash(fs))
# Set comprehension with filter
print(sorted({x ** 2 for x in range(5) if x > 1}))
# clear
s = {1, 2, 3}
s.clear()
print(len(s))
# Set edge cases: remove KeyError
s = {1, 2, 3}
print(len(s))
try:
    {1, 2}.remove(99)
except KeyError:
    print('caught')
