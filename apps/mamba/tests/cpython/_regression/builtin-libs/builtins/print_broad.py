# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# print() variations

# basic
print("hello")
print()  # empty
print("a", "b", "c")
print(1, 2, 3)
print("x", 1, [2, 3])

# sep / end
print("a", "b", "c", sep="")
print("a", "b", "c", sep="-")
print("a", "b", "c", sep=" | ")
print("hi", end="")
print(" there")
print("first", end="!")
print(" second", end="?")
print()  # newline

# combine sep + end
print("a", "b", "c", sep=",", end=";\n")

# print None
print(None)
print(None, None)

# print bool
print(True, False)

# print list/dict/tuple
print([1, 2, 3])
print({"a": 1})
print((1, 2, 3))

# print expressions
print(2 + 3)
print(len("hello"))

# print string with newlines
print("a\nb\nc")
print("tab\there")

# print escape sequences
print("\\n is newline")
print("quote: \"")
print("apos: '")
