# RUN: parse
# CPython 3.12 test_tuple: tuple operations

# Construction
t = ()
t = (1,)
t = (1, 2, 3)
t = tuple()
t = tuple([1, 2, 3])
t = tuple("hello")

# Indexing and slicing
t = (1, 2, 3, 4, 5)
x = t[0]
x = t[-1]
x = t[1:3]
x = t[::-1]

# Immutability (parse only)
t = (1, 2, 3)
# t[0] = 10  # would fail at runtime

# Methods
t = (1, 2, 1, 3, 1)
idx = t.index(1)
cnt = t.count(1)

# Operators
t = (1, 2) + (3, 4)
t = (0,) * 5
b = 1 in t
b = 10 not in t

# Tuple unpacking
a, b, c = (1, 2, 3)
(x, y) = (1, 2)
a, b = b, a  # swap

# Nested tuple
nested = ((1, 2), (3, 4))
x = nested[0][1]

# Named tuple usage (import syntax only)
from collections import namedtuple
Point = namedtuple("Point", ["x", "y"])

# len, min, max, sum
n = len(t)
mn = min(t)
mx = max(t)
s = sum(t)

# sorted (returns list)
lst = sorted((3, 1, 2))

# Bare tuple (no parens)
t = 1, 2, 3
t = 1,
