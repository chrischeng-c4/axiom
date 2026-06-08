# print options broad

# basic
print("hello")
print("a", "b", "c")
print()  # empty line
print("")  # empty string + default newline

# sep
print("a", "b", "c", sep="-")
print("a", "b", "c", sep="")
print("a", "b", "c", sep=", ")
print(1, 2, 3, sep=":")

# end
print("no newline", end="")
print(" continued")
print("line", end="!\n")
print("x", end="|")
print("y", end="|")
print("z")

# sep + end
print("a", "b", sep="-", end="!\n")
print(1, 2, 3, sep=".", end=";\n")

# mix types
print(1, "two", 3.0, True, None)
print([1, 2], {3, 4}, (5, 6))

# escaping
print("tab\there")
print("newline\ninside")
print("quote\"here")
print("backslash\\here")

# print in loop
for i in range(3):
    print("iter", i)

# print in function
def announce(name):
    print(">>", name)

announce("alice")
announce("bob")

# print with f-string
name = "world"
print(f"hello {name}")
print(f"pi = {3.14}")
print(f"{1+1} = two")

# print with format
print("x = {}".format(10))
print("{} + {} = {}".format(1, 2, 3))
print("{:05d}".format(42))

# repr vs str
print(str(42))
print(repr(42))
print(str("hi"))
print(repr("hi"))
