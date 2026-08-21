# bitwise ops broad

# AND
print(0b1100 & 0b1010)
print(0xFF & 0x0F)
print(0 & 0xFFFF)
print(0xFFFF & 0xFFFF)

# OR
print(0b1100 | 0b0011)
print(0 | 0xFF)
print(0xFF | 0xFF)

# XOR
print(0b1100 ^ 0b0110)
print(0xFF ^ 0xFF)
print(0xAA ^ 0x55)

# NOT
print(~0)
print(~-1)
print(~0b1010)

# shift left
print(1 << 0)
print(1 << 1)
print(1 << 8)
print(1 << 16)
print(3 << 4)

# shift right
print(256 >> 0)
print(256 >> 1)
print(256 >> 4)
print(256 >> 8)
print(1024 >> 10)

# combined
x = 0xFF
print(x & 0x0F)
print(x | 0x100)
print(x ^ 0xFF)
print(x << 4)
print(x >> 4)

# masking
def low_byte(n):
    return n & 0xFF

print(low_byte(0x1234))
print(low_byte(0xFFFF))
print(low_byte(0xAB))

# bit test
def has_bit(n, b):
    return (n >> b) & 1

print(has_bit(0b1010, 0))
print(has_bit(0b1010, 1))
print(has_bit(0b1010, 2))
print(has_bit(0b1010, 3))

# set bit
def set_bit(n, b):
    return n | (1 << b)

print(set_bit(0, 0))
print(set_bit(0, 3))
print(set_bit(0b1001, 1))
