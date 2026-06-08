# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: int.to_bytes / int.from_bytes returned None / raised
# AttributeError. Added direct implementations that honor length,
# byteorder ('big' / 'little'), and signed.

# to_bytes — positional byteorder
print((255).to_bytes(1, 'big'))
print((255).to_bytes(2, 'big'))
print((255).to_bytes(2, 'little'))
print((65535).to_bytes(2, 'big'))
print((0).to_bytes(4, 'big'))

# from_bytes — positional
print(int.from_bytes(b'\xff\xfe', 'big'))
print(int.from_bytes(b'\xff\xfe', 'little'))
print(int.from_bytes(b'\x00\x00\x00\x01', 'big'))

# Signed round-trip
signed_bytes = (-1).to_bytes(4, 'big', signed=True)
print(signed_bytes)
print(int.from_bytes(signed_bytes, 'big', signed=True))

# Unsigned from_bytes of the same bytes yields positive value
print(int.from_bytes(signed_bytes, 'big'))
