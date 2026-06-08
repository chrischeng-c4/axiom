# integer arithmetic edge cases broad

# basic
print(2 + 3)
print(10 - 4)
print(6 * 7)
print(100 // 3)
print(100 % 3)
print(2 ** 10)

# negative arithmetic
print(-5 + 3)
print(-5 * 3)
print(-10 // 3)
print(-10 % 3)
print(10 // -3)
print(10 % -3)

# large
print(2 ** 20)
print(1000 * 1000)
print(10 ** 6)
print(999 * 999)

# zero
print(0 + 0)
print(0 - 5)
print(0 * 100)
print(0 // 1)
print(0 % 5)
print(0 ** 5)

# identity
print(5 + 0)
print(5 * 1)
print(5 // 1)
print(5 ** 1)
print(5 ** 0)

# chained arithmetic
print(1 + 2 + 3 + 4 + 5)
print(1 * 2 * 3 * 4 * 5)
print(100 - 20 - 30 - 40)
print(1000 // 10 // 5)

# precedence
print(2 + 3 * 4)
print((2 + 3) * 4)
print(2 * 3 + 4)
print(2 ** 3 * 4)
print(2 * 3 ** 2)
print(10 - 3 - 2)
print(10 - (3 - 2))

# unary minus
print(-5)
print(--5)
print(- -5)
print(-(3 + 2))
print(-(-5))

# unary plus
print(+5)
print(+(-5))

# comparisons
print(5 == 5)
print(5 != 4)
print(5 < 10)
print(5 > 2)
print(5 <= 5)
print(5 >= 5)
print(5 == 5.0)

# modulo edge cases
print(7 % 1)
print(100 % 10)
print(1 % 7)

# pow with zero
print(0 ** 0)
print(1 ** 100)

# int overflow-free
print(2 ** 30)
print(2 ** 40)
print(3 ** 20)

# divmod
print(divmod(10, 3))
print(divmod(100, 7))
print(divmod(0, 5))

# abs
print(abs(-5))
print(abs(0))
print(abs(5))
print(abs(-100))

# min/max
print(min(1, 2))
print(min(1, 2, 3))
print(max(1, 2))
print(max(1, 2, 3))
print(min(-5, 0, 5))
print(max(-5, 0, 5))

# arithmetic in condition
for i in range(5):
    if i * 2 > 5:
        print(i)
