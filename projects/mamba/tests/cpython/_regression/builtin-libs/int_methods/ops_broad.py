# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# int methods ops broad

# bit_length
print((0).bit_length())
print((1).bit_length())
print((7).bit_length())
print((8).bit_length())
print((255).bit_length())
print((256).bit_length())
print((-1).bit_length())
print((-100).bit_length())

# abs / neg
print(abs(-5))
print(abs(5))
print(abs(0))

# int() conversions
print(int("42"))
print(int("100"))
print(int("-7"))
print(int(3.14))
print(int(-3.14))
print(int(True))
print(int(False))

# int with base
print(int("ff", 16))
print(int("101", 2))
print(int("77", 8))

# bool / truthiness
print(bool(0))
print(bool(1))
print(bool(-1))
print(bool(100))

# type name
print(type(5).__name__)
