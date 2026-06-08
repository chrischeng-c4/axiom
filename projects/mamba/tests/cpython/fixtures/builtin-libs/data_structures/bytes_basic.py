# bytes literal — basic ops

b = b"hello"
print(b)
print(len(b))
print(b[0])
print(b[-1])
print(b[1:4])

# iteration yields ints
for x in b:
    print(x)

# concatenation
print(b + b" world")

# repetition
print(b"ab" * 3)

# comparison
print(b == b"hello")
print(b == b"world")

# in operator
print(b"ell" in b)
print(b"xyz" in b)
print(104 in b)  # 'h'
print(99 in b)   # 'c'

# bytes() constructor from list
print(bytes([72, 101, 108, 108, 111]))

# bytes repr with non-printable bytes
print(bytes([0, 1, 2, 255]))
print(bytes([ord('a'), 0, ord('b')]))

# bytes(int) — zero-filled
print(bytes(3))
