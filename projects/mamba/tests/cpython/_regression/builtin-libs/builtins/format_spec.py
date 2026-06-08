# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# format() builtin with simple format specs

# Integer formats
print(format(42, "d"))
print(format(42, "b"))
print(format(42, "o"))
print(format(42, "x"))

# Float formats
print(format(3.14159, ".2f"))
print(format(3.14159, ".4f"))

# String alignment
print(format("hi", ">10"))
print(format("hi", "<10"))
