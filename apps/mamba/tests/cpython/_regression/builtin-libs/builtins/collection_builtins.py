# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: collection-building builtins (R5).
# list, tuple, set, dict, frozenset, min, max, sum, any, all

# list / tuple / set construction
print(list("hello"))
print(list(range(5)))
print(tuple([1, 2, 3]))
print(tuple("abc"))
print(sorted(set([3, 1, 4, 1, 5, 9, 2, 6])))
print(frozenset([1, 2, 3]) == frozenset([3, 2, 1]))

# dict
print(dict(a=1, b=2))
print(dict([("x", 10), ("y", 20)]))

# min / max
print(min(3, 1, 4, 1, 5))
print(max(3, 1, 4, 1, 5))
print(min([10, 20, 30]))
print(max([10, 20, 30]))
print(min("banana"))
print(max("banana"))

# sum
print(sum([1, 2, 3, 4, 5]))
print(sum(range(101)))
print(sum([1.5, 2.5, 3.0]))
print(sum([10, 20], 5))

# any / all
print(any([False, False, True]))
print(any([False, False, False]))
print(any([]))
print(all([True, True, True]))
print(all([True, False, True]))
print(all([]))
