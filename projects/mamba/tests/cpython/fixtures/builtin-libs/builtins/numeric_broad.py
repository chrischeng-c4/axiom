# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# numeric broad

# int operations
print(42)
print(-42)
print(abs(-17))
print(42 + 8)
print(42 - 8)
print(42 * 8)
print(42 // 8)
print(42 % 8)
print(42 / 8)
print(2 ** 10)
print(2 ** 16)

# int edge cases
print(0 // 7)
print(-7 // 3)
print(-7 % 3)
print(7 % -3)

# large ints
print(10 ** 15)
print(10 ** 18)

# bitwise
print(0b1100 & 0b1010)
print(0b1100 | 0b1010)
print(0b1100 ^ 0b1010)
print(~5)
print(8 << 3)
print(256 >> 4)

# float
print(3.14)
print(-2.5)
print(abs(-3.14))
print(round(3.7))
print(round(3.4))
print(round(3.14159, 2))
print(round(-2.5))

# conversion
print(int("42"))
print(int("-17"))
print(int("1010", 2))
print(int("ff", 16))
print(int("777", 8))
print(int(3.7))
print(int(-3.7))
print(float("3.14"))
print(float(42))
print(str(42))
print(str(3.14))

# boolean as int
print(True + True)
print(True * 5)
print(int(True))
print(int(False))
print(bool(1))
print(bool(0))
print(bool(""))
print(bool("x"))
print(bool([]))
print(bool([0]))

# comparison chains
print(1 < 2 < 3)
print(1 < 2 < 0)
print(1 == 1 == 1)
print(1 < 2 > 1)

# min/max
print(min(5, 3, 9, 1, 7))
print(max(5, 3, 9, 1, 7))
print(min([5, 3, 9]))
print(max([5, 3, 9]))
print(min(1.5, 2.5))
print(max(1.5, 2.5))
