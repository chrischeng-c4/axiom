# zip patterns broad

# basic zip
a = [1, 2, 3]
b = ["a", "b", "c"]
for x, y in zip(a, b):
    print(x, y)

# zip uneven (stops at shortest)
x = [1, 2, 3, 4, 5]
y = ["a", "b", "c"]
for p, q in zip(x, y):
    print(p, q)

# zip 3 iters
r = [1, 2, 3]
s = [10, 20, 30]
t = [100, 200, 300]
for i, j, k in zip(r, s, t):
    print(i, j, k)

# zip to list
li = list(zip([1, 2, 3], ["x", "y", "z"]))
print(len(li))
for item in li:
    print(item)

# zip with range
for i, v in zip(range(5), ["a", "b", "c", "d", "e"]):
    print(i, v)

# zip with enumerate (flat unpack)
items = ["apple", "banana", "cherry"]
weights = [1.0, 2.0, 3.0]
zipped = list(zip(items, weights))
for i in range(len(zipped)):
    print(i, zipped[i])

# zip sums
xs = [1, 2, 3, 4]
ys = [10, 20, 30, 40]
total = 0
for a, b in zip(xs, ys):
    total += a + b
print(total)

# zip produces tuples
pairs = list(zip([1, 2, 3], [4, 5, 6]))
print(pairs[0])
print(pairs[1])
print(pairs[2])

# zip of strings
for c1, c2 in zip("abc", "xyz"):
    print(c1, c2)

# zip accumulate into dict
keys = ["x", "y", "z"]
vals = [10, 20, 30]
d = {}
for k, v in zip(keys, vals):
    d[k] = v
print(sorted(d.items()))

# zip empty
print(list(zip([], [1, 2])))
print(list(zip([1, 2], [])))
print(list(zip([], [])))
