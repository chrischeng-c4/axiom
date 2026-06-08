# bytes <-> str bridge via encode/decode

# str.encode returns bytes
b = "hello".encode()
print(b)
print(type(b).__name__)

# roundtrip
print(b.decode())
print(b.decode() == "hello")

# encode non-ascii
print("café".encode())

# bytes.decode
print(b"world".decode())

# encode yields bytes that support bytes operations
b2 = "abc".encode()
print(b2 + b"xyz")
print(b2 * 2)
print(len(b2))
print(b2[0])

# iterate — ints
for x in "AB".encode():
    print(x)

# encode with default and explicit utf-8
print("ok".encode())
print("ok".encode("utf-8"))
