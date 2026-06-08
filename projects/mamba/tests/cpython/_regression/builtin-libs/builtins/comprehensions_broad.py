# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# comprehensions, enumerate, zip, dict/set/frozenset constructors

# enumerate with start
for i, v in enumerate(["a", "b", "c"], start=10):
    print(i, v)

# zip on different-length
print(list(zip([1, 2, 3], ["a", "b", "c", "d"])))

# zip with 3 iterables
print(list(zip([1, 2], ["a", "b"], [True, False])))

# dict comprehension with condition
print({k: v for k, v in [("a", 1), ("b", 2), ("c", 3)] if v > 1})

# nested comprehension
print([[i * j for j in range(3)] for i in range(3)])

# set comprehension
print(sorted({x * 2 for x in [1, 2, 3, 2, 1]}))

# tuple unpacking in comprehension
print([a + b for a, b in [(1, 2), (3, 4), (5, 6)]])

# conditional expression in comprehension
print([x if x > 0 else 0 for x in [-1, 2, -3, 4]])

# dict() constructor variants
print(dict())
print(sorted(dict(a=1, b=2).items()))
print(sorted(dict([("a", 1), ("b", 2)]).items()))

# set() constructor variants
print(sorted(set()))
print(sorted(set("abc")))
print(sorted(set([1, 2, 2, 3])))

# frozenset() constructor
print(sorted(frozenset([1, 2, 3, 2])))
