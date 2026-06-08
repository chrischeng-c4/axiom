# comprehension patterns deep

# basic list comp
sq = [x * x for x in range(5)]
print(sq)

# with condition
evens = [x for x in range(10) if x % 2 == 0]
print(evens)

# transform and filter
odd_cubes = [x * x * x for x in range(10) if x % 2 == 1]
print(odd_cubes)

# nested for
pairs = [(i, j) for i in range(3) for j in range(3)]
print(pairs)

# nested with condition
pairs_eq = [(i, j) for i in range(3) for j in range(3) if i == j]
print(pairs_eq)

pairs_diag = [(i, j) for i in range(3) for j in range(3) if i != j]
print(pairs_diag)

# flatten 2d list
mat = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [x for row in mat for x in row]
print(flat)

# extract column
col0 = [row[0] for row in mat]
col1 = [row[1] for row in mat]
print(col0)
print(col1)

# matrix transform
mat_d = [[x * 2 for x in row] for row in mat]
print(mat_d)

# dict comp
squares_d = {x: x * x for x in range(5)}
print(sorted(squares_d.items()))

# dict comp from pairs
pairs = [("a", 1), ("b", 2), ("c", 3)]
d = {k: v * 10 for k, v in pairs}
print(sorted(d.items()))

# set comp
s = {x % 3 for x in range(20)}
print(sorted(s))

# set comp with condition
s2 = {x for x in range(20) if x % 4 == 0}
print(sorted(s2))

# gen expr
g = (x * x for x in range(5))
print(list(g))

print(sum(x for x in range(10)))
print(sum(x * x for x in range(5)))

# comp over string
chars = [c for c in "hello"]
print(chars)

# comp with enumerate
idx = [(i, v) for i, v in enumerate(["a", "b", "c"])]
print(idx)

# comp with zip
zipped = [(a, b) for a, b in zip([1, 2, 3], ["x", "y", "z"])]
print(zipped)

# comp with range step
stepped = [x for x in range(0, 20, 3)]
print(stepped)

# conditional expression inside comp
signs = [1 if x > 0 else -1 if x < 0 else 0 for x in [-3, -1, 0, 1, 3]]
print(signs)

# string join result of comp
joined = ",".join([str(x) for x in range(5)])
print(joined)

# len of comp
print(len([x for x in range(100)]))
print(len([x for x in range(100) if x % 2 == 0]))

# max/min of comp
print(max(x * 2 for x in range(5)))
print(min(x * 2 for x in range(5)))

# any/all of comp
print(any(x > 10 for x in range(5)))
print(all(x >= 0 for x in range(5)))
