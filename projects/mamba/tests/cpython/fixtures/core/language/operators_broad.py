# operator-like patterns

# bit operations on ints
print(5 & 3)
print(5 | 3)
print(5 ^ 3)
print(~5)
print(5 << 2)
print(20 >> 2)

# bit ops chain
x = 0
x |= 1
x |= 2
x |= 4
print(x)
print(x & 0b011)
print(x & 0b100)

# comparison
print(1 < 2 < 3)
print(1 < 2 > 0)
print(5 == 5 == 5)
print(1 < 2 < 1)

# is / is not
print(None is None)
print(None is not None)
a = []
b = []
print(a is b)
print(a is a)

# in / not in
print(3 in [1, 2, 3])
print(4 in [1, 2, 3])
print("a" in "abc")
print("x" not in "abc")
print("k" in {"k": 1})

# boolean short-circuit
print(True and False)
print(True or False)
print(0 or 5)
print(1 and 2)
print(None or "default")

# unary
print(-5)
print(+5)
print(-(-5))
print(not True)
print(not False)
print(not 0)
print(not [])

# modulo / power
print(17 % 5)
print(-17 % 5)
print(2 ** 10)
print(2 ** 0)
print(0 ** 0)
