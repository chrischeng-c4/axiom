# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list.sort and sorted() with key= and reverse=

xs = [3, 1, 4, 1, 5, 9, 2, 6]

# sort in-place
ys = xs[:]
ys.sort()
print(ys)

# sort reverse
ys = xs[:]
ys.sort(reverse=True)
print(ys)

# sorted() — returns new list
print(sorted(xs))
print(sorted(xs, reverse=True))

# sort with key
words = ["apple", "fig", "banana", "cherry"]
print(sorted(words, key=len))

# sort by lambda
nums = [-3, 1, -4, 1, 5, -9, 2]
print(sorted(nums, key=lambda n: abs(n)))

# Sort tuples by second element
pairs = [("a", 3), ("b", 1), ("c", 2)]
print(sorted(pairs, key=lambda p: p[1]))

# Sort strings case-insensitive
names = ["Bob", "alice", "Carol"]
print(sorted(names, key=lambda s: s.lower()))

# Stable sort
items = [("a", 1), ("b", 1), ("c", 2)]
print(sorted(items, key=lambda p: p[1]))