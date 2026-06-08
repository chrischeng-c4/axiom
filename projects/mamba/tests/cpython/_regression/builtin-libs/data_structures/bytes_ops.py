# Data structures conformance: bytes and bytearray (R2.5).
# All methods: decode, fromhex, hex, split, strip, replace, find, startswith, endswith
# Mutable bytearray ops

# --- bytes construction ---
b1 = bytes([72, 101, 108, 108, 111])  # b"Hello"
b2 = b"Hello, World!"
print(b1)
print(b2)

# decode
print(b1.decode("utf-8"))
print(b2.decode())

# fromhex / hex
b3 = bytes.fromhex("48656c6c6f")
print(b3)
print(b1.hex())

# find
print(b2.find(b"World"))
print(b2.find(b"xyz"))

# startswith / endswith
print(b2.startswith(b"Hello"))
print(b2.endswith(b"World!"))
print(b2.startswith(b"World"))

# split
parts = b"a,b,c".split(b",")
print(parts)

# strip / lstrip / rstrip
print(b"  hello  ".strip())
print(b"  hello  ".lstrip())
print(b"  hello  ".rstrip())

# replace
print(b"hello world".replace(b"world", b"python"))

# count
print(b"banana".count(b"an"))

# upper / lower (bytes don't have these — test what's there)
# in / contains
print(b"ell" in b"Hello")

# len
print(len(b1))
print(len(b2))

# --- bytearray construction ---
ba1 = bytearray([72, 101, 108, 108, 111])
ba2 = bytearray(b"Hello")
print(ba1)
print(ba2)

# mutable: item assignment
ba1[0] = 74   # 'J'
print(ba1)

# append / extend
ba3 = bytearray(b"abc")
ba3.append(100)   # 'd'
print(ba3)
ba3.extend(b"ef")
print(ba3)

# pop
ba4 = bytearray(b"xyz")
v = ba4.pop()
print(v)
print(ba4)

# bytearray decode
print(ba2.decode("utf-8"))

# bytearray hex
print(ba2.hex())

# bytearray reverse
ba5 = bytearray(b"abc")
ba5.reverse()
print(ba5)

# equality
print(bytes([72, 101]) == b"He")
print(bytearray(b"abc") == b"abc")
