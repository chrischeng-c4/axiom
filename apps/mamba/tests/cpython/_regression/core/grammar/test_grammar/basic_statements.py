# RUN: parse
# CPython 3.12 test_grammar: basic statement coverage

# Simple assignments
x = 1
y = 2
z = x + y

# Augmented assignments
x += 1
x -= 1
x *= 2
x //= 2
x **= 2
x %= 3
x &= 0xFF
x |= 0x01
x ^= 0x10
x >>= 1
x <<= 1

# Multiple assignment targets
a = b = c = 0

# Tuple unpacking
a, b = 1, 2
a, b, c = 1, 2, 3
(a, b) = (1, 2)
[a, b] = [1, 2]

# Delete
del x
del a, b

# Pass
pass

# Assert
assert True
assert x == 1, "x should be 1"

# Global / nonlocal
def outer():
    x = 10
    def inner():
        nonlocal x
        x = 20
    inner()

def use_global():
    global z
    z = 99
