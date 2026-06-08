# enumerate + zip deep broad

# enumerate basic
for i, v in enumerate(["a", "b", "c"]):
    print(i, v)

# enumerate with start
for i, v in enumerate(["a", "b", "c"], start=1):
    print(i, v)

for i, v in enumerate(["x", "y"], start=100):
    print(i, v)

# enumerate string
for i, c in enumerate("abc"):
    print(i, c)

# enumerate tuple
for i, v in enumerate((10, 20, 30)):
    print(i, v)

# enumerate range
for i, v in enumerate(range(5)):
    print(i, v)

# enumerate + accumulate
total = 0
for i, v in enumerate([10, 20, 30, 40]):
    total = total + v
    print(i, total)

# zip basic
for a, b in zip([1, 2, 3], ["a", "b", "c"]):
    print(a, b)

# zip short circuit
for a, b in zip([1, 2, 3, 4, 5], ["x", "y"]):
    print(a, b)

# zip 3 iterables
for a, b, c in zip([1, 2], [10, 20], [100, 200]):
    print(a, b, c)

# zip w/ string
for a, b in zip("ab", "12"):
    print(a, b)

# list(zip)
print(list(zip([1, 2, 3], [4, 5, 6])))
print(list(zip([1, 2], [3, 4], [5, 6])))

# zip empty
print(list(zip([], [1, 2])))
print(list(zip([1, 2], [])))

# tuple unpack in for
data = [(1, "a"), (2, "b"), (3, "c")]
for k, v in data:
    print(k, v)
