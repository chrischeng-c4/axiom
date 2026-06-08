# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string predicates broad (is-methods)

# isalpha
print("hello".isalpha())
print("abc123".isalpha())
print("".isalpha())
print("hello world".isalpha())

# isdigit
print("12345".isdigit())
print("abc".isdigit())
print("".isdigit())
print("12.5".isdigit())
print("  ".isdigit())

# isalnum
print("abc123".isalnum())
print("hello".isalnum())
print("12345".isalnum())
print("".isalnum())
print("hello world".isalnum())
print("hi!".isalnum())

# isspace
print(" ".isspace())
print("\t".isspace())
print("\n".isspace())
print("   \t\n  ".isspace())
print("".isspace())
print("a ".isspace())

# isupper / islower
print("HELLO".isupper())
print("Hello".isupper())
print("hello".islower())
print("Hello".islower())
print("123".isupper())
print("".isupper())

# istitle
print("Hello World".istitle())
print("Hello world".istitle())
print("hello".istitle())
print("".istitle())

# startswith/endswith variants
print("hello".startswith("he"))
print("hello".startswith("HE"))
print("hello".endswith("lo"))
print("hello".endswith("LO"))

# starts/ends with empty
print("hello".startswith(""))
print("hello".endswith(""))

# starts/ends with tuple
print("hello".startswith(("he", "ab")))
print("hello".startswith(("ab", "xy")))
print("hello".endswith(("lo", "ab")))
print("hello".endswith(("ab", "xy")))

# startswith with start/end
print("hello world".startswith("world", 6))
print("hello world".startswith("hello", 0, 5))

# chained predicates
print("abc".isalpha() and "abc".islower())
print("ABC".isalpha() and "ABC".isupper())
print("123".isdigit() and "123".isalnum())

# via variable
name = "alice"
if name.isalpha():
    print("all letters")

code = "abc_123"
print(code.isalnum())  # False (_ not alnum)

# length after method chain
print(len("  hi  ".strip()))
print(len("Hello".lower()))

# casefold
print("HELLO".casefold())
print("Hello".casefold())
print("ABC123".casefold())

# chained case
print("HeLLo".lower().upper())
print("HELLO".lower().capitalize())

# title case
print("hello world".title())
print("python is fun".title())
print("".title())

# swapcase
print("Hello World".swapcase())
print("PyThOn".swapcase())

# center with fill
print("hi".center(10))
print("hi".center(10, "*"))
print("hi".center(11, "-"))

# rjust/ljust
print("42".rjust(5))
print("42".rjust(5, "0"))
print("42".ljust(5))
print("42".ljust(5, "."))

# zfill
print("42".zfill(5))
print("-42".zfill(5))
print("abc".zfill(5))
print("42".zfill(2))
print("42".zfill(1))
