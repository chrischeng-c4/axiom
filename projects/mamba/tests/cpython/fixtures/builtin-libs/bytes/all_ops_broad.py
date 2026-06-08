# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# bytes broad (working subset)

# basic construction
b1 = b"hello"
print(b1)
print(len(b1))

# bytes from string with encoding
b3 = "world".encode("utf-8")
print(b3)

# bytes decode
print(b"hello".decode())

# indexing (returns int)
print(b"hello"[0])
print(b"hello"[-1])

# slicing (forward)
print(b"hello world"[:5])
print(b"hello world"[6:])

# concat
print(b"abc" + b"def")

# multiply
print(b"ab" * 3)

# membership
print(b"ell" in b"hello")
print(b"xyz" in b"hello")

# methods
print(b"  hi  ".strip())
print(b"a,b,c".split(b","))
print(b"-".join([b"a", b"b", b"c"]))
print(b"hello".startswith(b"he"))
print(b"hello".endswith(b"lo"))
print(b"abc".replace(b"b", b"X"))

# comparison
print(b"abc" == b"abc")
print(b"abc" != b"abd")

# hex / fromhex
print(b"\x00\x01\x02".hex())
print(bytes.fromhex("010203"))
