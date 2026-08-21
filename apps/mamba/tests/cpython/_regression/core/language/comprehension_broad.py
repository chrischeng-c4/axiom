print([x * 2 for x in range(5)])
print([x * x for x in range(5) if x % 2 == 0])
print([x + y for x in range(3) for y in range(3)])
print([(x, y) for x in range(3) for y in range(3) if x != y])

print({k: v for k, v in [("a", 1), ("b", 2), ("c", 3)]})
print({x: x * x for x in range(5)})
print({x: x * 2 for x in range(6) if x % 2 == 0})

print(sorted({x % 3 for x in range(10)}))
print(sorted({c for c in "hello world" if c != " "}))

print(sum(x * x for x in range(10)))
print(all(x > 0 for x in [1, 2, 3, 4]))
print(all(x > 0 for x in [1, -1, 3]))
print(any(x > 100 for x in [1, 2, 3]))
print(any(x < 0 for x in [1, -1, 3]))

# nested comp
m = [[i * 3 + j for j in range(3)] for i in range(3)]
print(m)

# flatten
flat = [x for row in m for x in row]
print(flat)

# double loop with condition
pairs = [(i, j) for i in range(4) for j in range(4) if i < j]
print(pairs)

# dict comp from zip
keys = ["a", "b", "c"]
vals = [1, 2, 3]
print({k: v for k, v in zip(keys, vals)})

# enumerate in comp
print([(i, c) for i, c in enumerate("xyz")])
