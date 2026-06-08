# Tuple edge cases: empty constructor, concat, repeat, from iterable, count/index, hash, comparisons
print(tuple())
print((1,) + (2, 3))
print((0,) * 3)
print(tuple([1, 2]))
t = (1, 2, 2, 3)
print(t.count(2))
print(t.index(2))
print(t.count(99))
# Hash consistency
print(hash((1, 2)) == hash((1, 2)))
# Comparison chains
print((1, 2) < (1, 3))
print((1, 2) <= (1, 2))
print((1, 2) >= (1, 2))
