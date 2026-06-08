# zip() with multiple iterables and mismatched lengths

# Two equal-length
xs = [1, 2, 3]
ys = ["a", "b", "c"]
for t in zip(xs, ys):
    print(t)

# Three iterables
zs = [10, 20, 30]
for a, b, c in zip(xs, ys, zs):
    print(a, b, c)

# Shorter drops trailing — Python semantics
short = [1, 2]
long = [10, 20, 30, 40]
for a, b in zip(short, long):
    print(a, b)

# Zip empty yields nothing
for t in zip([], [1, 2, 3]):
    print("never")
print("empty done")

# Zip with generator
def gen(n):
    i = 0
    while i < n:
        yield i
        i = i + 1

for a, b in zip(gen(4), ["w", "x", "y"]):
    print(a, b)

# list(zip(...))
print(list(zip([1, 2], [3, 4])))

# Unpacking via zip(*pairs) style
pairs = [(1, "a"), (2, "b"), (3, "c")]
keys = []
vals = []
for k, v in pairs:
    keys.append(k)
    vals.append(v)
print(keys)
print(vals)
