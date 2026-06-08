# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string replace/translate/various ops broad

# replace
s = "hello world"
print(s.replace("world", "python"))
print(s.replace("l", "L"))
print(s.replace(" ", "_"))
print(s.replace("xyz", "!"))
print(s.replace("o", "0", 1))
print(s.replace("o", "0", 2))
print(s.replace("o", ""))
print("".replace("a", "b"))

# replace with limit 0 / negative
print("aaaa".replace("a", "b", 0))
print("aaaa".replace("a", "b"))
print("aaaa".replace("a", "b", 2))

# chain replacements
print("a,b,c".replace(",", "; ").replace("a", "A"))

# expandtabs
print("a\tb".expandtabs(4))
print("\tx".expandtabs(2))

# encode/decode bytes basic
b = "hello".encode("utf-8")
print(type(b).__name__)
print(len(b))
print(b.decode("utf-8"))

# split with maxsplit
print("a,b,c,d,e".split(",", 2))
print("a,b,c,d,e".split(",", 0))
print("a,b,c,d,e".split(","))

# splitlines
print("a\nb\nc".splitlines())
print("a\nb\n".splitlines())
print("".splitlines())

# join variations
print("-".join(["1", "2", "3"]))
print(" | ".join(["one", "two"]))
print("".join(["x", "y", "z"]))

# strip with chars
print("..hi..".strip("."))
print("--x--".strip("-"))
print("abcxxcba".strip("abc"))

# lstrip / rstrip with chars
print(".hi".lstrip("."))
print("hi.".rstrip("."))
print("..hi..".lstrip("."))
print("..hi..".rstrip("."))

# str * int
print("ab" * 3)
print("*" * 10)
print("" * 5)
print("x" * 0)

# int in str repetition by variable
n = 4
print("-" * n)
print("=" * n)

# contains operator
print("ell" in "hello")
print("xy" in "hello")
print("" in "hello")
print("hello" in "hello")

# string equality
print("abc" == "abc")
print("abc" == "abd")
print("" == "")
print("abc" != "abd")

# ordinal comparison
print("a" < "b")
print("b" < "a")
print("apple" < "banana")
print("apple" < "apricot")

# upper/lower idempotent
print("HELLO".upper())
print("hello".lower())
print("".upper())
print("".lower())

# contains specific types
print("1" in "123")
print("2" in "123")
print("0" in "123")

# string formatted output
x = 42
print(f"value={x}")
print(f"text")
print(f"{x * 2}")

# count substring
print("banana".count("a"))
print("banana".count("na"))
print("banana".count("xyz"))
print("".count("a"))
print("aaaa".count("aa"))

# find/rfind
print("banana".find("a"))
print("banana".rfind("a"))
print("banana".find("x"))
print("banana".rfind("x"))
print("banana".find("an"))
print("banana".rfind("an"))
