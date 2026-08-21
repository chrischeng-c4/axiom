# comprehension v2 broad

# list comp with ternary
print([x if x > 0 else -x for x in [-2, -1, 0, 1, 2]])
print(["even" if x % 2 == 0 else "odd" for x in range(5)])

# list comp nested 3d
print([[[z for z in range(2)] for y in range(2)] for x in range(2)])

# list comp flatten
matrix = [[1, 2], [3, 4], [5, 6]]
print([v for row in matrix for v in row])

# dict comp with value transform
print({x: x * x for x in range(5)})
print({c: ord(c) for c in "abc"})

# dict comp from zip
keys = ["a", "b", "c"]
vals = [1, 2, 3]
print({k: v for k, v in zip(keys, vals)})

# dict comp with condition
print({x: x * 2 for x in range(10) if x % 2 == 0})

# set comp unique filter
print(sorted({x % 5 for x in range(20)}))

# set comp with operation
print(sorted({x * 2 for x in [1, 2, 3, 2, 1]}))

# set comp on string
print(sorted({c for c in "hello world"}))

# comp with nested iteration
print([(a, b) for a in [1, 2] for b in [10, 20]])

# comp nested with condition
print([(a, b) for a in range(3) for b in range(3) if a != b])

# comp reads outer var
threshold = 3
print([x for x in range(10) if x > threshold])

# empty comp
print([x for x in []])
print({x: 1 for x in []})

# generator expr w/ sum
print(sum(x * 2 for x in range(5)))
print(sum(x for x in range(10)))
