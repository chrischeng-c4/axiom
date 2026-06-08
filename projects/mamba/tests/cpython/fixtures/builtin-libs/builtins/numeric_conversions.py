# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# hex / oct / bin / chr / ord / divmod / abs / pow / round

# hex / oct / bin
print(hex(255))
print(hex(0))
print(hex(-255))
print(oct(8))
print(oct(0))
print(bin(10))
print(bin(0))
print(bin(-10))

# chr / ord
print(chr(65))
print(chr(97))
print(chr(0x03B1))  # α
print(ord('A'))
print(ord('a'))
print(ord('α'))

# divmod (int)
print(divmod(17, 5))
print(divmod(-17, 5))
print(divmod(17, -5))
print(divmod(0, 5))

# abs
print(abs(-3))
print(abs(-3.14))
print(abs(3))
print(abs(0))

# pow
print(pow(2, 10))
print(pow(2, 10, 1000))
print(pow(3, 0))
print(pow(2, -1))

# round (banker's rounding)
print(round(3.14159, 2))
print(round(0.5))
print(round(1.5))
print(round(2.5))
print(round(-0.5))
print(round(-1.5))
print(round(3.5))
print(round(12345, -2))
