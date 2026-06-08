# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# tuple broad ops

t = (1, 2, 3, 4, 5)

# basic access
print(t[0])
print(t[-1])
print(len(t))

# slicing
print(t[1:3])
print(t[::2])
print(t[2:])
print(t[:2])

# methods
print(t.count(3))
print((1, 2, 3, 2, 1, 2).count(2))
print(t.index(3))

# unpacking
a, b, c, d, e = t
print(a, b, c, d, e)

# partial unpack with *
first, *rest = t
print(first, rest)

*init, last = t
print(init, last)

head, *mid, tail = t
print(head, mid, tail)

# single element tuple
single = (42,)
print(single)

# empty tuple
empty = ()
print(empty)
print(len(empty))
print(bool(empty))

# tuple + concat
print((1, 2) + (3, 4))
print((1, 2) * 3)
print(1 in (1, 2, 3))
print(99 not in (1, 2, 3))

# tuple from iterable
print(tuple([1, 2, 3]))
print(tuple("abc"))
print(tuple(range(5)))

# swap via tuple
x, y = 10, 20
x, y = y, x
print(x, y)

# return multiple
def minmax(xs):
    return min(xs), max(xs)

mn, mx = minmax([3, 1, 4, 1, 5, 9, 2, 6])
print(mn, mx)
