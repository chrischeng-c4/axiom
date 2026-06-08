# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string .format() method (not f-string)

# positional
print("Hello, {}!".format("world"))
print("{} + {} = {}".format(1, 2, 3))

# numbered positional
print("{0} and {1}, but also {0}".format("apple", "banana"))
print("{1} before {0}".format("B", "A"))

# named
print("{name} is {age}".format(name="Alice", age=30))

# mix (note: not all Python does — skip)

# format spec
print("{:5}".format(42))
print("{:05}".format(42))
print("{:.2f}".format(3.14159))

# alignment
print("[{:<10}]".format("hi"))
print("[{:>10}]".format("hi"))
print("[{:^10}]".format("hi"))
