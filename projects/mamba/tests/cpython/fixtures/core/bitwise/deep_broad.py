# bitwise ops deep broad

# AND
print(0b1100 & 0b1010)
print(0xff & 0x0f)
print(0xff & 0xf0)
print(0 & 0xff)
print(0xff & 0xff)

# OR
print(0b1100 | 0b1010)
print(0xff | 0x0f)
print(0x0f | 0xf0)
print(0 | 0xff)

# XOR
print(0b1100 ^ 0b1010)
print(0xff ^ 0xff)
print(0xff ^ 0x00)
print(0b1111 ^ 0b1010)

# NOT
print(~0)
print(~1)
print(~(-1))
print(~0xff)

# shifts
print(1 << 0)
print(1 << 4)
print(1 << 8)
print(1 << 16)
print(256 >> 4)
print(256 >> 8)
print(0xff >> 4)
print(0xffff >> 4)

# combining
print((1 << 3) | (1 << 5))
print(0b1111_0000 & 0b1100_1100)
print(0b1010 ^ 0b0101)

# augmented
x = 0b1100
x &= 0b1010
print(x)

x = 0b1100
x |= 0b0011
print(x)

x = 0b1111
x ^= 0b1010
print(x)

x = 1
x <<= 4
print(x)

x = 256
x >>= 4
print(x)

# bit-level identities
print(5 | 0 == 5)
print(5 & 0xffff == 5)
print(5 ^ 0 == 5)
print(5 ^ 5 == 0)

# priority: & < | in same expr — verify with parens
print((0b1100 | 0b0011) & 0b1010)
print(0b1100 | (0b0011 & 0b1010))

# hex and binary literals round-trip
print(hex(255))
print(hex(4096))
print(hex(0))
print(bin(5))
print(bin(0))
print(bin(255))
print(oct(8))
print(oct(64))
