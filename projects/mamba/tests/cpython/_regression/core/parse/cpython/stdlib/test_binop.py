# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_binop.py — syntax constructs only.


# --- Arithmetic operators ---

1 + 2
10 - 3
4 * 5
20 / 4
17 // 3
17 % 3
2 ** 10


# --- Arithmetic with different literal types ---

1 + 2.0
2.5 * 3
10.0 / 3
7.5 // 2.0
7.5 % 2.5
2.0 ** 0.5

1 + 2j
3.0 + 4j
(1 + 2j) * (3 - 4j)
(2 + 3j) / (1 + 1j)
(1 + 0j) ** 2

0xF + 0o7
0b1010 + 0xFF
0o777 - 0x1FF


# --- Negative operands ---

-1 + 2
3 + -4
-5 * -6
-10 / -3
-10 // -3
-10 % 3
10 % -3


# --- Bitwise operators ---

0xFF & 0x0F
0x0F | 0xF0
0xFF ^ 0x0F
1 << 8
256 >> 4
0b1100 & 0b1010
0b1100 | 0b1010
0b1100 ^ 0b1010


# --- Chained arithmetic ---

1 + 2 + 3
10 - 3 - 2
2 * 3 * 4
100 / 2 / 5
100 // 3 // 2
1 + 2 * 3
(1 + 2) * 3
2 ** 3 ** 2


# --- Operator precedence ---

# Power is right-associative
2 ** 3 ** 2

# Unary minus vs power
-2 ** 2
(-2) ** 2

# Multiplication before addition
1 + 2 * 3 + 4
1 - 2 * 3 + 4

# Division variants
10 / 2 + 3
10 // 2 + 3
10 % 3 + 1

# Shift vs addition
1 + 2 << 3
(1 + 2) << 3
8 >> 1 + 1
8 >> (1 + 1)

# Bitwise precedence
1 | 2 & 3
(1 | 2) & 3
1 ^ 2 & 3
1 ^ (2 & 3)
1 | 2 ^ 3
(1 | 2) ^ 3

# Mixed arithmetic and bitwise
(1 + 2) & 0xFF
(10 * 3) | 0x01
(100 // 7) ^ 0x0F


# --- Large number operations ---

10 ** 20 + 10 ** 19
10 ** 100 * 2
999999999999999999 + 1
2 ** 64 - 1
2 ** 128


# --- Operations in expressions ---

x = 10
y = 3
z = x + y
z = x - y
z = x * y
z = x / y
z = x // y
z = x % y
z = x ** y
z = x & y
z = x | y
z = x ^ y
z = x << y
z = x >> y


# --- Binary ops in function args ---

def add(a, b):
    return a + b

def sub(a, b):
    return a - b

def mul(a, b):
    return a * b

result = add(1 + 2, 3 * 4)
result = sub(10 - 5, 2 ** 3)
result = mul(0xFF & 0x0F, 1 << 4)


# --- Binary ops in conditions ---

if 1 + 1 == 2:
    pass

if 10 // 3 > 2:
    pass

if 0xFF & 0x0F == 0x0F:
    pass

while 2 ** 0 == 1:
    break


# --- Binary ops in comprehensions ---

squares = [x ** 2 for x in range(10)]
sums = [x + y for x in range(3) for y in range(3)]
masked = [x & 0xFF for x in [256, 512, 1024]]
shifted = [1 << n for n in range(8)]


# --- Binary ops in ternary ---

x = 10
y = (x + 1) if x > 5 else (x - 1)
z = (x * 2) if (x & 1) == 0 else (x * 3 + 1)


# --- Operator dunder methods ---

class Vec:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __add__(self, other):
        return Vec(self.x + other.x, self.y + other.y)

    def __sub__(self, other):
        return Vec(self.x - other.x, self.y - other.y)

    def __mul__(self, scalar):
        return Vec(self.x * scalar, self.y * scalar)

    def __rmul__(self, scalar):
        return Vec(scalar * self.x, scalar * self.y)

    def __truediv__(self, scalar):
        return Vec(self.x / scalar, self.y / scalar)

    def __floordiv__(self, scalar):
        return Vec(self.x // scalar, self.y // scalar)

    def __mod__(self, scalar):
        return Vec(self.x % scalar, self.y % scalar)

    def __pow__(self, exp):
        return Vec(self.x ** exp, self.y ** exp)

    def __and__(self, other):
        return Vec(self.x & other.x, self.y & other.y)

    def __or__(self, other):
        return Vec(self.x | other.x, self.y | other.y)

    def __xor__(self, other):
        return Vec(self.x ^ other.x, self.y ^ other.y)

    def __lshift__(self, n):
        return Vec(self.x << n, self.y << n)

    def __rshift__(self, n):
        return Vec(self.x >> n, self.y >> n)

v1 = Vec(1, 2)
v2 = Vec(3, 4)
v3 = v1 + v2
v4 = v1 - v2
v5 = v1 * 3
v6 = 3 * v1
v7 = v2 / 2
v8 = v2 // 2
v9 = v2 % 3
v10 = v1 ** 2
v11 = v1 & v2
v12 = v1 | v2
v13 = v1 ^ v2
v14 = v1 << 1
v15 = v2 >> 1


# --- Reflected operators ---

class MyNum:
    def __init__(self, val):
        self.val = val

    def __radd__(self, other):
        return MyNum(other + self.val)

    def __rsub__(self, other):
        return MyNum(other - self.val)

    def __rmul__(self, other):
        return MyNum(other * self.val)

    def __rtruediv__(self, other):
        return MyNum(other / self.val)

    def __rfloordiv__(self, other):
        return MyNum(other // self.val)

    def __rmod__(self, other):
        return MyNum(other % self.val)

    def __rpow__(self, other):
        return MyNum(other ** self.val)

n = MyNum(5)
r1 = 10 + n
r2 = 10 - n
r3 = 10 * n
r4 = 10 / n
r5 = 10 // n
r6 = 10 % n
r7 = 2 ** n


# --- Nested binary ops ---

a = 1
b = 2
c = 3
d = 4

result = (a + b) * (c - d)
result = ((a + b) * c) ** d
result = (a | b) & (c ^ d)
result = ((a << b) + c) >> d
result = a + b * c ** d
