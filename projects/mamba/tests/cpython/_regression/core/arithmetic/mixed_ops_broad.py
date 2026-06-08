# arithmetic mixed ops patterns broad

# order of operations
print(1 + 2 * 3)
print((1 + 2) * 3)
print(2 + 3 * 4 - 1)
print(10 - 2 - 3)
print(10 - (2 - 3))
print(100 / 10 / 2)
print(2 ** 3 ** 2)  # right-associative: 2**(3**2) = 2**9 = 512

# unary minus
print(-5)
print(-(-5))
print(--5)
print(-0)
print(-1 + 2)
print(-(1 + 2))

# power
print(2 ** 0)
print(2 ** 1)
print(2 ** 10)
print(3 ** 4)
print(10 ** 5)
print(0 ** 0)
print(0 ** 5)
print(1 ** 100)

# floor div
print(17 // 3)
print(17 // 5)
print(100 // 7)
print(-17 // 3)
print(17 // -3)
print(-17 // -3)

# modulo
print(17 % 3)
print(100 % 7)
print(-17 % 3)
print(17 % -3)

# mixed types
print(1 + 2.0)
print(3.0 * 2)
print(10.0 // 3)
print(10 % 3.0)

# bitwise
print(0b1100 & 0b1010)
print(0b1100 | 0b1010)
print(0b1100 ^ 0b1010)
print(~0b1010)
print(1 << 4)
print(256 >> 3)

# absolute value
print(abs(-5))
print(abs(5))
print(abs(0))
print(abs(-3.14))
print(abs(3.14))

# sum
print(sum([1, 2, 3, 4, 5]))
print(sum([1, 2, 3], 100))
print(sum([]))
print(sum(range(10)))
print(sum([1.5, 2.5, 3.0]))

# pow function
print(pow(2, 10))
print(pow(3, 4))
print(pow(2, 0))
print(pow(10, 3))

# integer divmod-style
print(divmod(17, 5))
print(divmod(20, 4))
print(divmod(100, 7))
print(divmod(-17, 5))
