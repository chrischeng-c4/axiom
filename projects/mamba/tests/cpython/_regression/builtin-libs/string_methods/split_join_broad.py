# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string split / join / strip patterns broad

# split basic
print("a,b,c".split(","))
print("a b c".split())
print("one|two|three".split("|"))

# split with maxsplit
print("a,b,c,d".split(",", 1))
print("a,b,c,d".split(",", 2))
print("a,b,c,d".split(",", 0))

# split empty string
print("".split(","))
print("".split())

# split no match
print("hello".split(","))

# split by whitespace
print("  one   two   three  ".split())
print("a\tb\nc".split())

# splitlines
print("line1\nline2\nline3".splitlines())
print("a\nb\nc".splitlines())
print("".splitlines())
print("no newline".splitlines())

# join
print(",".join(["a", "b", "c"]))
print("-".join(["1", "2", "3"]))
print("".join(["a", "b", "c"]))
print(" ".join(["hello", "world"]))

# join single elt
print(",".join(["only"]))

# join empty
print(",".join([]))

# join with numbers (must be strings)
print(",".join([str(x) for x in [1, 2, 3]]))

# strip
print("  hello  ".strip())
print("xxhelloxx".strip("x"))
print("...hi...".strip("."))

# lstrip / rstrip
print("  hi  ".lstrip())
print("  hi  ".rstrip())
print("xxhi".lstrip("x"))
print("hixx".rstrip("x"))

# strip empty
print("".strip())
print("abc".strip())

# strip mixed chars
print("<<<hi>>>".strip("<>"))

# split+join round-trip
parts = "hello,world,foo".split(",")
print(",".join(parts))
print("|".join(parts))
print("".join(parts))

# count
print("banana".count("a"))
print("banana".count("an"))
print("hello".count("l"))
print("hello".count("x"))
print("".count("a"))

# find / rfind
print("hello".find("l"))
print("hello".rfind("l"))
print("hello".find("x"))
print("hello".rfind("x"))
