# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# tuple patterns broad

# basic construction
t = (1, 2, 3)
print(t)
print(len(t))

# indexing
t5 = (10, 20, 30, 40, 50)
print(t5[0])
print(t5[-1])
print(t5[2])

# slicing
print(t5[:3])
print(t5[::-1])

# concatenation
a = (1, 2)
b = (3, 4)
print(a + b)

# count / index
t2 = (1, 2, 1, 3, 1, 4)
print(t2.count(1))
print(t2.index(2))

# iterate
total = 0
for v in t5:
    total += v
print(total)

# unpacking
x, y, z = (100, 200, 300)
print(x)
print(y)
print(z)

# swap
a, b = 1, 2
a, b = b, a
print(a, b)

# function return as tuple
def minmax(lst):
    lo = min(lst)
    hi = max(lst)
    return lo, hi

lo, hi = minmax([3, 1, 4, 1, 5, 9])
print(lo)
print(hi)

# nested
nested = ((1, 2), (3, 4), (5, 6))
print(nested[0])
print(nested[1][1])
for pair in nested:
    print(pair[0], pair[1])
