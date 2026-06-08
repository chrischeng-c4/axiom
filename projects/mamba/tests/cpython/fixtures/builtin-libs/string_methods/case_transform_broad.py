# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string case / transform patterns broad

# upper/lower
print("hello".upper())
print("HELLO".lower())
print("MiXeD".upper())
print("MiXeD".lower())
print("".upper())
print("".lower())

# capitalize
print("hello".capitalize())
print("HELLO".capitalize())
print("hello world".capitalize())
print("a".capitalize())
print("".capitalize())

# title
print("hello world".title())
print("python is fun".title())
print("already Title".title())

# swapcase
print("Hello World".swapcase())
print("abc".swapcase())
print("ABC".swapcase())
print("HelloWorld".swapcase())

# casefold
print("HELLO".casefold())
print("Hello".casefold())
print("abc".casefold())

# chained transforms
print("hello".upper().lower())
print("hello world".title().upper())

# per-char loop
s = "Hello World"
upper_count = 0
lower_count = 0
for c in s:
    if c.isupper():
        upper_count += 1
    elif c.islower():
        lower_count += 1
print(upper_count, lower_count)

# apply on each word
words = ["hello", "world", "foo"]
print([w.upper() for w in words])
print([w.capitalize() for w in words])
print([w[::-1] for w in words])

# replace transform
text = "hello world"
print(text.replace("hello", "hi"))
print(text.replace("o", "0"))
print(text.replace(" ", "_"))
print(text.replace("xyz", "ABC"))

# replace count
print("aaaaa".replace("a", "b", 2))
print("aaaaa".replace("a", "b", 0))
print("aaaaa".replace("a", "b", 100))

# pad + strip
padded = "  hello  "
print(padded)
print(padded.strip())
print(padded.lstrip())
print(padded.rstrip())
print(len(padded))
print(len(padded.strip()))

# encode/decode roundtrip via bytes (if supported) - skip, use str-only

# chain replace
s2 = "hello.world.foo.bar"
print(s2.replace(".", "/"))
print(s2.replace(".", " ").title())

# compare case-insensitive
print("Hello".lower() == "hello")
print("Hello".lower() == "HELLO".lower())
print("abc".upper() == "ABC")
