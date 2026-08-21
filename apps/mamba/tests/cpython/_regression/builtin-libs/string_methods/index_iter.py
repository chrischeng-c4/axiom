# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string iteration/indexing

s = "hello"
for c in s:
    print(c)

# indexing with negative
print(s[0])
print(s[-1])
print(s[-2])

# in operator
print("h" in s)
print("z" in s)
print("ell" in s)

# iteration with index (simulated)
for i in range(len(s)):
    print(i, s[i])

# enumerate
for i, c in enumerate(s):
    print(i, c)

# reversed string
for c in reversed(s):
    print(c)

# char codes
print(ord("a"))
print(ord("A"))
print(ord("0"))

# chr
print(chr(65))
print(chr(97))
print(chr(48))

# string * int
print("abc" * 3)
print("-" * 10)
print("" * 5)
print("x" * 0)

# string concatenation
print("hello" + " " + "world")

# comparison
print("abc" < "abd")
print("abc" == "abc")
print("abc" > "abb")

# sorted characters
print(sorted("dcba"))

# list-of-chars from str
print(list("hello"))

# str(int)
print(str(42))
print(str(-17))
print(str(3.14))
print(str(True))
print(str(False))
print(str([1, 2]))
print(str((1, 2)))

# int from string
print(int("42"))
print(int("-17"))
