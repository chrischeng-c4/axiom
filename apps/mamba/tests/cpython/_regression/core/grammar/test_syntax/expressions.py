# RUN: parse
# CPython 3.12 test_syntax: expression coverage

# Arithmetic
x = 1 + 2
x = 3 - 1
x = 4 * 5
x = 10 / 3
x = 10 // 3
x = 10 % 3
x = 2 ** 10
x = -x
x = +x
x = ~x

# Comparison chaining
result = 1 < 2 < 3
result = 1 < 2 and 2 < 3
result = 1 == 1 != 2

# Boolean operators
x = True and False
x = True or False
x = not True

# Bitwise
x = 0xFF & 0x0F
x = 0x01 | 0x10
x = 0xFF ^ 0x0F
x = 1 << 4
x = 16 >> 2

# Walrus operator
if (n := 10) > 5:
    pass

data = [1, 2, 3, 4, 5]
while chunk := data[:2]:
    data = data[2:]

# Ternary
x = 1 if True else 0

# Starred expressions
a, *b, c = [1, 2, 3, 4, 5]

# Attribute access
import sys
v = sys.version

# Subscript
lst = [1, 2, 3]
x = lst[0]
x = lst[-1]
x = lst[1:3]
x = lst[::2]
x = lst[1:4:2]

# Call expressions
def f(*args, **kwargs): pass
f(1, 2, 3)
f(1, *[2, 3])
f(a=1, **{"b": 2})
f(1, 2, key=3)

# Comprehensions
squares = [x**2 for x in range(10)]
evens = [x for x in range(20) if x % 2 == 0]
nested = [x * y for x in range(5) for y in range(5)]

# Dict comprehension
d = {k: v for k, v in zip("abc", [1, 2, 3])}

# Set comprehension
s = {x**2 for x in range(10)}

# Generator expression
total = sum(x**2 for x in range(10))

# Lambda
fn = lambda x, y=0: x + y
