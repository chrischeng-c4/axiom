# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: functional builtins (R1.9).
# Tests map, filter, sorted, reversed, zip, enumerate.

# map
result = list(map(len, ["hello", "hi", "hey"]))
print(result)

# filter
evens = list(filter(lambda x: x % 2 == 0, range(10)))
print(evens)

# sorted
words = ["banana", "apple", "cherry"]
print(sorted(words))

# reversed
print(list(reversed([1, 2, 3, 4, 5])))

# zip
names = ["a", "b", "c"]
values = [1, 2, 3]
print(list(zip(names, values)))

# enumerate
for i, v in enumerate(["x", "y", "z"]):
    print(i, v)

# sum
print(sum([1, 2, 3, 4, 5]))
print(sum(range(10)))

# abs
print(abs(5))
print(abs(0))
