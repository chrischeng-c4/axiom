# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string case + strip methods broad

# upper / lower
print("hello".upper())
print("HELLO".lower())
print("Hello World".upper())
print("Hello World".lower())
print("".upper())
print("abc123".upper())
print("ABC123".lower())

# title
print("hello world".title())
print("HELLO WORLD".title())
print("hello-world".title())
print("a b c".title())

# capitalize
print("hello".capitalize())
print("HELLO".capitalize())
print("hello world".capitalize())
print("".capitalize())

# swapcase
print("Hello World".swapcase())
print("ABC def".swapcase())
print("".swapcase())

# strip variants
print("  hello  ".strip())
print("  hello  ".lstrip())
print("  hello  ".rstrip())
print("xxhelloxx".strip("x"))
print("xxhelloxx".lstrip("x"))
print("xxhelloxx".rstrip("x"))
print("\t\nhello\t\n".strip())

# no-op strip
print("hello".strip())
print("hello".lstrip())
print("hello".rstrip())

# strip custom chars
print("***abc***".strip("*"))
print(".,abc.,".strip(".,"))

# is-methods
print("abc".isalpha())
print("abc123".isalpha())
print("123".isdigit())
print("abc".isdigit())
print("abc123".isalnum())
print("abc!".isalnum())
print("   ".isspace())
print("abc".isspace())
print("Hello".istitle())
print("HELLO".isupper())
print("hello".islower())
print("HELLO".islower())
print("hello".isupper())

# startswith / endswith
print("hello".startswith("hel"))
print("hello".startswith("abc"))
print("hello".endswith("llo"))
print("hello".endswith("abc"))
print("hello world".startswith("hello"))
print("hello world".endswith("world"))

# center / ljust / rjust
print("hi".center(10))
print("hi".center(10, "-"))
print("hi".ljust(10))
print("hi".ljust(10, "."))
print("hi".rjust(10))
print("hi".rjust(10, "."))

# zfill
print("42".zfill(5))
print("-5".zfill(5))
print("abc".zfill(6))
print("".zfill(3))
