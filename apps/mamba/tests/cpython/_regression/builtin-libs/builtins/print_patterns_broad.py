# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# print patterns broad

# single arg
print("hello")
print(42)
print(3.14)
print(True)
print(None)

# multi args default sep
print(1, 2, 3)
print("a", "b", "c")
print(1, "two", 3.0)

# sep kwarg
print(1, 2, 3, sep=",")
print("a", "b", "c", sep="-")
print(1, 2, 3, sep="")
print(1, 2, 3, sep=" | ")

# end kwarg
print("hello", end="!\n")
print("foo", end=" ")
print("bar")
print("x", end="")
print("y", end="")
print("z")

# sep + end
print("a", "b", sep="+", end=";\n")

# print empty
print()
print("---")
print()

# print list/tuple/dict
print([1, 2, 3])
print((1, 2, 3))
print({"a": 1, "b": 2})

# print nested
print([[1, 2], [3, 4]])
print({"items": [1, 2, 3]})

# print with str operations
s = "hello"
print(s.upper())
print(s + " world")
print(s * 2)

# print with math
print(2 + 3)
print(10 * 5)
print(100 / 4)

# print in loop
for i in range(3):
    print("item", i)

# print conditional
x = 10
print("positive" if x > 0 else "non-positive")

# print with join
print(", ".join(["a", "b", "c"]))

# print f-string simple
name = "alice"
age = 30
print(f"{name}")
print(f"{age}")
print(f"{name} is {age}")

# print escape
print("line1\nline2")
print("tab\there")
print("quote\"here")
print("back\\slash")
