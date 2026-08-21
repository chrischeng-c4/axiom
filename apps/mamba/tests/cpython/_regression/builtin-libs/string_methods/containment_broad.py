# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string containment / search broad

# in operator
print("a" in "abc")
print("b" in "abc")
print("c" in "abc")
print("d" in "abc")
print("ab" in "abc")
print("bc" in "abc")
print("ac" in "abc")
print("abc" in "abc")
print("abcd" in "abc")
print("" in "abc")

# not in
print("a" not in "abc")
print("d" not in "abc")
print("" not in "abc")

# case sensitive
print("A" in "abc")
print("a" in "ABC")

# multi-char search
print("hello" in "hello world")
print("world" in "hello world")
print("llo wo" in "hello world")
print("xyz" in "hello world")

# empty strings
print("" in "")
print("a" in "")
print("" in "a")

# char in list
print("a" in ["a", "b", "c"])
print("d" in ["a", "b", "c"])

# str in list of str
print("hello" in ["hello", "world"])
print("foo" in ["hello", "world"])

# iterate chars
for c in "hello":
    print(c)

# string length
print(len("hello"))
print(len(""))
print(len("a"))
print(len("hello world"))

# string indexing
s = "hello"
print(s[0])
print(s[1])
print(s[-1])
print(s[-2])

# forward slicing
print(s[:3])
print(s[1:4])
print(s[2:])
print(s[:])

# string repetition
print("abc" * 3)
print("-" * 10)
print("xy" * 0)
print("xy" * 1)

# concatenation
print("hello" + " " + "world")
print("" + "x")
print("x" + "")

# bool from contains
if "ell" in "hello":
    print("found")
else:
    print("not found")

# count substrings
print("hello".count("l"))
print("hello".count("ll"))
print("aaaa".count("a"))
print("aaaa".count("aa"))
print("".count("x"))
