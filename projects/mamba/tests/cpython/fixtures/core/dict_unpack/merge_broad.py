# dict unpack / merge broad

# basic merge
a = {"x": 1, "y": 2}
b = {"z": 3, "w": 4}
merged = {**a, **b}
print(sorted(merged.items()))

# override
c = {"x": 10, "y": 20}
d = {"y": 99, "z": 30}
over = {**c, **d}
print(sorted(over.items()))

# with literal
e = {"a": 1}
lit = {**e, "b": 2, "c": 3}
print(sorted(lit.items()))

# before literal
f = {"x": 10}
pre = {"y": 20, **f}
print(sorted(pre.items()))

# empty source
g = {}
h = {"a": 1}
print(sorted({**g, **h}.items()))
print(sorted({**h, **g}.items()))

# triple merge
t1 = {"a": 1}
t2 = {"b": 2}
t3 = {"c": 3}
tri = {**t1, **t2, **t3}
print(sorted(tri.items()))

# self-merge
s = {"a": 1}
ss = {**s, **s}
print(sorted(ss.items()))
