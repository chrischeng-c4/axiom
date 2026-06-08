# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
print("hello".upper())
print("HELLO".lower())
print("".upper())
print("123".upper())

print("hello.py".endswith((".py", ".txt")))
print("hello.py".endswith((".txt", ".md")))
print("hello".startswith(("he", "hi")))
print("hello".startswith(("ha", "hi")))

print("abcabc".find("c"))
print("abcabc".rfind("c"))
print("abc".find("x"))
print("abc".rfind("x"))
print("abc".find("a", 1))

print("abcabc".index("c"))
# "abcabc".rindex("c") returns None unexpectedly

print("hello".replace("l", "L"))
print("hello".replace("ll", "XY"))
print("aaaa".replace("a", "b", 2))

print("a b c".split())
print("a  b   c".split())
print("a,b,,c".split(","))
print("a,b,c".split(",", 1))

print("  hi  ".strip())
print("xxhixx".strip("x"))
print("  hi  ".lstrip())
print("  hi  ".rstrip())
print("XXhello".lstrip("X"))
print("helloXX".rstrip("X"))

print("-".join(["a", "b", "c"]))
print(",".join(["x", "y"]))
print("".join(["a", "b", "c"]))

print("abc".isalpha())
print("abc123".isalnum())
print("123".isdigit())
print("abc".isupper())
print("ABC".isupper())
print("ABC".islower())
print("abc".islower())
print("   ".isspace())
print(" a ".isspace())
print("".isdigit())

print("hello".encode())
print(b"hello".decode())

print(len("hello"))
print(len(""))

print("ab" * 3)

print("ll" in "hello")
print("xy" in "hello")
