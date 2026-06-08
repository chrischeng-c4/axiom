# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
print(b"hello")
print(len(b"hello"))
b = b"hello"
print(b[0])
print(b[-1])
print(b[1:4])
print(b[:2])

print(b"foo" + b"bar")
print(b"ab" * 3)

print(b"  hello  ".strip())
print(b"a-b-c".replace(b"-", b"_"))

print(b"ll" in b"hello")
print(b"xy" in b"hello")

print(b"hello".startswith(b"he"))
print(b"hello".endswith(b"lo"))

print(b"hello".decode())
print(b"hello".decode("utf-8"))

print(b"abc" == b"abc")
print(b"abc" == b"abd")
