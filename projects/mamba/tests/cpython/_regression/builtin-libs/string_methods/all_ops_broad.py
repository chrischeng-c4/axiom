# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string broader ops

s = "Hello, World!"

# case-based
print(s.upper())
print(s.lower())
print("hello".capitalize())
print("HELLO".capitalize())

# find / index / rfind
print(s.find("o"))
print(s.rfind("o"))
print(s.find("X"))
print(s.find("l", 3))
print(s.find("l", 3, 5))
print(s.index("o"))

# startswith / endswith
print("hello".startswith("he"))
print("hello".endswith("lo"))
print("hello".startswith(("ha", "he")))
print("hello".endswith(("no", "lo")))

# split / rsplit
print("a,b,c,d".split(","))
print("a,b,c,d".split(",", 2))
print("  a b  c ".split())

# replace
print("aaa".replace("a", "b"))
print("aaa".replace("a", "b", 2))
print("abcabc".replace("bc", "XY"))

# join
print(",".join(["a", "b", "c"]))
print("-".join([str(i) for i in range(5)]))
print("".join("abc"))

# strip
print("  hi  ".strip())
print("--abc--".strip("-"))
print("   x".lstrip())
print("x   ".rstrip())

# checks
print("abc".isalpha())
print("a1b".isalpha())
print("123".isdigit())
print("1.5".isdigit())
print("abc123".isalnum())
print("abc!".isalnum())
print("   ".isspace())
print("abc".isspace())
print("HELLO".isupper())
print("hello".islower())

# slicing
print(s[:5])
print(s[7:])
print(s[::-1])
print(s[::2])
print(s[-6:-1])

# concat / multiply
print("ab" + "cd")
print("x" * 5)
print("abc"[0])
print("abc"[-1])

# iterable
print(list("abc"))
print(len("hello"))
