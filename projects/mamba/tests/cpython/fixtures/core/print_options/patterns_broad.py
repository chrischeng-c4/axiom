# print options broad

# sep
print("a", "b", "c")
print("a", "b", "c", sep="-")
print(1, 2, 3, sep=",")
print("x", "y", sep="")

# end
print("hi", end="")
print("!")
print("line1", end="\n")
print("line2")

# sep + end
print("a", "b", sep="-", end="!\n")

# multiple args of various types
print(1, 2.5, "s", True, None)

# single arg
print("solo")

# empty
print()
print("after-empty")
