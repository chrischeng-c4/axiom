# Numeric literal forms — int underscores (PEP 515), hex, binary,
# octal, and scientific notation. Float underscores and underscore-
# adjacent decimal points are a known parser gap and omitted.

# Underscores in decimal ints
print(1_000_000)
print(1_000_000 + 1)
print(0xFF_FF)
print(0x1_0000)
print(0b1_000_0000)

# Hex
print(0xFF)
print(0xff)
print(0x10)
print(0x0)

# Binary
print(0b1010)
print(0b0)
print(0b1111_1111)

# Octal
print(0o777)
print(0o10)
print(0o0)

# Mix across bases in one expression
x = 0xFF + 0b10
print(x)

# Scientific notation (floats)
print(1e3)
print(1.5e2)
print(2e-3)
