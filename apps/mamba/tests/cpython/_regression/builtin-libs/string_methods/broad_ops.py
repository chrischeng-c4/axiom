# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# str methods broad coverage

# center / ljust / rjust
print("abc".center(9))
print("abc".center(9, "*"))
print("abc".ljust(6, "-"))
print("abc".rjust(6, "-"))
print("abc".center(2))  # shorter than len

# zfill
print("42".zfill(5))
print("-42".zfill(5))
print("0x".zfill(5))
print("abcdef".zfill(3))  # longer than width

# partition / rpartition
print("a,b,c".partition(","))
print("a,b,c".rpartition(","))
print("no-commas".partition(","))

# splitlines
print("a\nb\nc".splitlines())
print("a\nb\nc\n".splitlines())
print("\n".splitlines())
print("".splitlines())
print("a\nb\nc".splitlines(True))

# expandtabs
print("a\tb".expandtabs())
print("a\tb".expandtabs(4))

# casefold
print("Hello".casefold())
print("ß".casefold())

# swapcase
print("Hello World".swapcase())
print("aB3".swapcase())

# title
print("hello world".title())
print("HELLO WORLD".title())

# count
print("abcabc".count("a"))
print("abcabc".count("bc"))
print("aaaa".count("aa"))  # non-overlapping
print("aaaa".count("a", 1, 3))

# removeprefix / removesuffix
print("unhappy".removeprefix("un"))
print("unhappy".removeprefix("xx"))
print("beautiful".removesuffix("ful"))
print("beautiful".removesuffix("xx"))
