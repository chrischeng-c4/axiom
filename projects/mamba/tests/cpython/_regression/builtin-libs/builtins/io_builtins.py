# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: hash, id, hex, oct, bin, bool, bytes, bytearray (R5).

# bool
print(bool(0))
print(bool(1))
print(bool(-1))
print(bool(0.0))
print(bool(0.1))
print(bool(""))
print(bool("hello"))
print(bool([]))
print(bool([0]))
print(bool(None))

# hex / oct / bin
print(hex(255))
print(hex(0))
print(hex(-255))
print(oct(8))
print(oct(64))
print(bin(10))
print(bin(0))
print(bin(-5))

# bytes / bytearray
b: bytes = bytes([65, 66, 67])
print(b)
print(len(b))
ba: bytearray = bytearray([72, 101, 108, 108, 111])
print(ba)
print(len(ba))

# hash — only check type (value is implementation-defined for some types)
print(type(hash(42)).__name__)
print(type(hash("hello")).__name__)
print(type(hash((1, 2, 3))).__name__)
print(hash(42) == hash(42))
print(hash(True) == hash(1))
print(hash(False) == hash(0))
