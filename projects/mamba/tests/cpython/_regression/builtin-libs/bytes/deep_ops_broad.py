# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# bytes deep broad

# construction
b = b"hello"
print(b)
print(b"")
print(b"a\nb")
print(b"\x48\x65\x6c\x6c\x6f")  # Hello

# indexing → int
print(b[0])  # 104
print(b[1])
print(b[-1])

# len
print(len(b))
print(len(b""))

# slicing (forward only — reverse was broken)
print(b[1:3])
print(b[:2])
print(b[2:])
print(b[1:])

# concatenation
print(b"a" + b"b")
print(b"hi " + b"there")

# multiplication
print(b"ab" * 3)
print(b"!" * 5)

# membership
print(b"e" in b)
print(b"xx" in b)
print(b"llo" in b)
print(b"e" not in b"xyz")

# bytes from str via encode
print("hi".encode())
print("hello".encode("ascii"))
print("abc".encode("utf-8"))

# decode
print(b"hello".decode())
print(b"hello".decode("ascii"))
print(b"abc".decode("utf-8"))

# strip / lstrip / rstrip
print(b"  hi  ".strip())
print(b"  hi  ".lstrip())
print(b"  hi  ".rstrip())

# split / join
print(b"a,b,c".split(b","))
print(b",".join([b"a", b"b", b"c"]))

# startswith/endswith
print(b"hello".startswith(b"he"))
print(b"hello".startswith(b"lo"))
print(b"hello".endswith(b"lo"))
print(b"hello".endswith(b"he"))

# replace
print(b"a-b-c".replace(b"-", b"+"))
print(b"abcabc".replace(b"bc", b"XY"))

# equality
print(b"abc" == b"abc")
print(b"abc" != b"abd")

# hex() on bytes
print(b"abc".hex())
print(b"\x00\xff\x80".hex())

# fromhex classmethod
print(bytes.fromhex("61626263"))

# bytes + literal-int-iter (like list constructor)
bs = bytes([65, 66, 67])
print(bs)
