# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string methods deeper broad

# center
print("hi".center(10))
print("hi".center(10, "*"))
print("hi".center(3))

# ljust / rjust
print("hi".ljust(6))
print("hi".ljust(6, "."))
print("hi".rjust(6))
print("hi".rjust(6, "."))

# zfill
print("42".zfill(5))
print("-7".zfill(5))
print("abc".zfill(6))
print("abc".zfill(2))

# splitlines
print("a\nb\nc".splitlines())
print("a\nb\nc\n".splitlines())
print("".splitlines())
print("one line".splitlines())

# title
print("hello world".title())
print("python programming".title())

# swapcase
print("Hello World".swapcase())

# casefold
print("Hello".casefold())

# count
print("mississippi".count("s"))
print("mississippi".count("ss"))
print("aaaa".count("aa"))
print("abc".count("d"))

# expandtabs
print("a\tb".expandtabs())
print("a\tb".expandtabs(4))

# lstrip / rstrip with specific chars
print("  hello  ".lstrip())
print("  hello  ".rstrip())
print("***hi***".lstrip("*"))
print("***hi***".rstrip("*"))

# partition / rpartition
print("a=b=c".partition("="))
print("a=b=c".rpartition("="))
print("no-eq".partition("="))
print("no-eq".rpartition("="))

# removeprefix / removesuffix (3.9+)
print("preifx_word".removeprefix("preifx_"))
print("word_suffix".removesuffix("_suffix"))
print("noprefix".removeprefix("bad_"))
print("nosuffix".removesuffix("_bad"))
