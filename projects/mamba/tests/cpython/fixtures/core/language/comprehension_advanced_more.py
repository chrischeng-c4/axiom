# list/tuple/dict comprehension advanced

# list comp with condition
print([x for x in range(10) if x % 2 == 0])
print([x * x for x in range(5)])

# nested comp
print([(i, j) for i in range(3) for j in range(2)])
print([[i + j for j in range(3)] for i in range(3)])

# if/else in comp
print([x if x > 0 else -x for x in [-2, -1, 0, 1, 2]])

# dict comp
print({x: x * x for x in range(5)})
print({k: v for k, v in [("a", 1), ("b", 2), ("c", 3)]})
print({x: x for x in range(5) if x % 2 == 0})

# set comp
print({x % 3 for x in range(10)})

# generator expr
print(sum(x * x for x in range(5)))
print(max(x for x in [3, 1, 4, 1, 5]))
print(list(x * 2 for x in range(4)))
print(any(x > 10 for x in [1, 2, 3]))
print(all(x > 0 for x in [1, 2, 3]))
print(any(x > 10 for x in [1, 20, 3]))

# tuple unpacking
pairs = [(1, "a"), (2, "b"), (3, "c")]
print([k for k, v in pairs])
print([v for k, v in pairs])
print({k: v for k, v in pairs})

# enumerate in comp
print([(i, v) for i, v in enumerate("abc")])

# zip in comp
print([(a, b) for a, b in zip([1, 2, 3], ["x", "y", "z"])])
