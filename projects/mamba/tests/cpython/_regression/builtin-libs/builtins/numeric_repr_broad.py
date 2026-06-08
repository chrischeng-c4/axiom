# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# bin/oct/hex
print(bin(10))
print(bin(0))
print(bin(255))
print(oct(8))
print(oct(0))
print(oct(64))
print(hex(255))
print(hex(0))
print(hex(16))

# negative
print(bin(-5))
print(hex(-255))
print(oct(-8))

# divmod
print(divmod(17, 5))
print(divmod(-17, 5))
print(divmod(17, -5))
print(divmod(100, 10))

# pow with 3 args
print(pow(2, 10))
print(pow(2, 10, 1000))
print(pow(3, 100, 7))

# abs
print(abs(-5))
print(abs(5))
print(abs(-3.14))
print(abs(0))

# round
print(round(3.7))
print(round(3.2))
print(round(-3.7))
print(round(3.14159, 2))
print(round(100, -1))

# min/max kw
print(min(3, 1, 4, 1, 5))
print(max([9, 2, 7, 3]))
print(min("hello"))
print(max("hello"))
