# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# int methods deep broad

# arithmetic
print(2 + 3)
print(10 - 3)
print(4 * 5)
print(20 / 4)
print(7 // 2)
print(7 % 2)
print(2 ** 10)
print(2 ** 30)
print(2 ** 63)  # big

# division
print(7 / 2)
print(-7 // 2)
print(7 // -2)
print(-7 // -2)

# modulo signs
print(7 % 3)
print(-7 % 3)
print(7 % -3)
print(-7 % -3)

# abs / neg / pos
print(abs(-42))
print(abs(42))
print(-(-5))
print(+5)

# int construction
print(int(3.7))
print(int(-3.7))
print(int(True))
print(int(False))
print(int("42"))
print(int("  -7 "))
print(int("0"))
print(int("0", 10))
print(int("101", 2))
print(int("ff", 16))
print(int("777", 8))

# bit_length
print((1).bit_length())
print((2).bit_length())
print((4).bit_length())
print((255).bit_length())
print((256).bit_length())
print((0).bit_length())

# hex / oct / bin on int
print(hex(255))
print(hex(0))
print(hex(16))
print(bin(5))
print(bin(0))
print(oct(8))
print(oct(0))

# comparisons
print(5 < 10)
print(5 > 10)
print(5 == 5)
print(5 != 10)
print(5 >= 5)
print(5 <= 5)

# int divisibility
print(10 % 2 == 0)
print(15 % 3 == 0)
print(7 % 2 == 0)

# chained arith
print((1 + 2) * (3 + 4))
print((1 + 2) ** 2)

# large values
print(2 ** 50)
print(10 ** 10)
print(12345 * 67890)
