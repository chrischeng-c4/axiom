# star unpacking broad

# basic tuple assignment
a, *rest = [1, 2, 3, 4]
print(a, rest)

*init, last = [1, 2, 3, 4]
print(init, last)

head, *mid, tail = [1, 2, 3, 4, 5]
print(head, mid, tail)

# empty middle
h, *m, t = [1, 2]
print(h, m, t)

# empty rest
a2, *r2 = [1]
print(a2, r2)

# from tuple
x, *y = (10, 20, 30)
print(x, y)

# from string
c1, *cr = "hello"
print(c1, cr)

# dict unpack
d1 = {"a": 1, "b": 2}
d2 = {"c": 3}
merged = {**d1, **d2}
print(sorted(merged.items()))

# star in function call
def fsum(x, y, z):
    return x + y + z

args = [1, 2, 3]
print(fsum(*args))
