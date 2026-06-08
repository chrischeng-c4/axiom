# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# str.format() with format specs — positional, indexed, keyword, format specifiers
print("{}+{}={}".format(1, 2, 3))
print("{0}{1}{0}".format("ab", "cd"))
print("{name}".format(name="world"))
x = 42
print(f"{x}")
print(f"{x:05d}")
print("{:.2f}".format(3.14159))
print("{:>10}".format("hi"))
print("{:<10}".format("hi"))
