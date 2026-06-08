# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: sequence functions (R5).
# len, range, sorted, reversed, enumerate, zip, map, filter

# len
print(len([1, 2, 3]))
print(len("hello"))
print(len((1, 2)))
print(len({}))
print(len({1, 2, 3}))

# range
print(list(range(5)))
print(list(range(2, 7)))
print(list(range(0, 10, 3)))
print(list(range(10, 0, -2)))

# sorted
print(sorted([3, 1, 4, 1, 5, 9, 2]))
print(sorted([3, 1, 4], reverse=True))
print(sorted(["banana", "apple", "cherry"]))

# reversed
print(list(reversed([1, 2, 3, 4, 5])))
print(list(reversed(range(5))))

# enumerate
for i, v in enumerate(["a", "b", "c"]):
    print(i, v)
for i, v in enumerate(["x", "y"], start=10):
    print(i, v)

# zip
for a, b in zip([1, 2, 3], ["a", "b", "c"]):
    print(a, b)
print(list(zip([1, 2], [3, 4], [5, 6])))

# map
print(list(map(str, [1, 2, 3])))
print(list(map(abs, [-1, -2, 3, -4])))

# filter
print(list(filter(None, [0, 1, False, 2, "", "a"])))
print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
