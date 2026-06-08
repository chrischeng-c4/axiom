# int/float methods broad

# int methods
print((10).bit_length())
print((255).bit_length())
print((0).bit_length())
print((1).bit_length())
print((1024).bit_length())

# int / float
print(int("123"))
print(int("  42 "))
print(int("-7"))
print(int("0"))

# int from float
print(int(3.9))
print(int(-3.9))
print(int(0.5))

# float conversions
print(float(1))
print(float("3.14"))
print(float("0.0"))
print(float("-2.5"))

# arithmetic
print(10 / 3)
print(10 // 3)
print(-10 // 3)
print(10 % 3)
print(-10 % 3)

# float truncation
x = 3.7
print(int(x))

# float abs
print(abs(3.14))
print(abs(-3.14))

# round
print(round(3.14))
print(round(3.5))
print(round(4.5))
print(round(-3.5))
print(round(3.14159, 2))

# bool arith
print(True + 1)
print(False * 10)
print(True + True + True)
print(int(True))
print(int(False))

# int from base
print(int("ff", 16))
print(int("101", 2))
print(int("777", 8))
