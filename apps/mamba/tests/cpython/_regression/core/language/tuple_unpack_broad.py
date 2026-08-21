# tuple unpacking patterns broad

# basic
a, b = 1, 2
print(a, b)

# 3-tuple
x, y, z = 10, 20, 30
print(x, y, z)

# unpack from tuple var
t = (100, 200, 300)
a, b, c = t
print(a, b, c)

# unpack from list
l = [1, 2, 3]
a, b, c = l
print(a, b, c)

# assign multi from tuple literal
x, y = (100, 200)
print(x, y)

# star unpack
a, *rest = [1, 2, 3, 4, 5]
print(a, rest)

*init, last = [1, 2, 3, 4, 5]
print(init, last)

first, *mid, last = [1, 2, 3, 4, 5]
print(first, mid, last)

# unpack in for
pairs = [(1, "a"), (2, "b"), (3, "c")]
for num, letter in pairs:
    print(num, letter)

# unpack items from enumerate
idx_items = []
for i, v in enumerate(["x", "y", "z"]):
    idx_items.append((i, v))
print(idx_items)

# unpack into dict context
d = {"a": 1, "b": 2, "c": 3}
for k, v in sorted(d.items()):
    print(k, "->", v)
