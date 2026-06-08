
# String methods
s = "hello world"
print(s.upper())
print(s.lower())
print(s.capitalize())
print(s.title())
print(s.strip())
print("  hello  ".strip())
print("  hello  ".lstrip())
print("  hello  ".rstrip())
# split and join
print("a,b,c".split(","))
print(",".join(["x", "y", "z"]))
# replace
print("hello world".replace("world", "python"))
# find
print("hello".find("ll"))
print("hello".find("xyz"))
# startswith / endswith
print("hello".startswith("hel"))
print("hello".endswith("llo"))
# count
print("banana".count("an"))
# format
print("Hello, {}!".format("world"))
print("{0} + {1} = {2}".format(1, 2, 3))
# f-string
name = "Bob"
age = 25
print(f"{name} is {age} years old")
print(f"{2 + 3}")
print(f"{'hello':>10}")
# len
print(len("hello"))
print(len(""))
