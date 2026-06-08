# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# chr / ord / conversion patterns broad

# chr basic
print(chr(65))
print(chr(97))
print(chr(48))
print(chr(32))
print(chr(33))

# ord basic
print(ord("A"))
print(ord("a"))
print(ord("0"))
print(ord(" "))
print(ord("!"))

# roundtrip
for c in "ABC":
    print(ord(c), chr(ord(c)))

# alphabet loop
for i in range(65, 75):
    print(chr(i))

# number digit loop
for i in range(48, 58):
    print(chr(i))

# uppercase via ord/chr
def upper_via_ord(c):
    o = ord(c)
    if 97 <= o <= 122:
        return chr(o - 32)
    return c

print(upper_via_ord("a"))
print(upper_via_ord("z"))
print(upper_via_ord("A"))
print(upper_via_ord("!"))

# hex/oct/bin
print(hex(255))
print(hex(16))
print(hex(0))
print(oct(8))
print(oct(64))
print(oct(0))
print(bin(5))
print(bin(10))
print(bin(0))

# int from hex string
print(int("ff", 16))
print(int("10", 16))
print(int("FF", 16))

# int from binary string
print(int("101", 2))
print(int("1111", 2))

# int from octal string
print(int("777", 8))
print(int("10", 8))

# convert int to str in bases
print(hex(256))
print(bin(256))
print(oct(256))
