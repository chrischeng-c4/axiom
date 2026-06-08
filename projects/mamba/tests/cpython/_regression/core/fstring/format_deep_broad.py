# string formatting deep broad

# format() basic
print("{}".format(42))
print("{}".format("hi"))
print("{} and {}".format(1, 2))
print("{0} {1} {0}".format("a", "b"))

# format() named
print("{name} is {age}".format(name="Alice", age=30))

# format() width/align
print("|{:5}|".format(42))
print("|{:<5}|".format(42))
print("|{:>5}|".format(42))
print("|{:^5}|".format("x"))

# fill char
print("{:*<5}".format("x"))
print("{:*>5}".format("x"))
print("{:*^5}".format("x"))

# numeric zero-pad
print("{:05d}".format(42))

# float
print("{:.3f}".format(3.14159))
print("{:.0f}".format(3.7))
print("{:8.2f}".format(3.14))

# f-string basics
x = 10
print(f"x={x}")
print(f"{x}")
print(f"{x!r}")
print(f"{x:5}")
print(f"{x:05d}")
print(f"{3.14:.2f}")
print(f"{'hi':*^5}")

# nested f-string expressions
a = 3
print(f"{a + 2}")
print(f"{a * 2 + 1}")

# f-string with method
s = "hello"
print(f"{s.upper()}")
print(f"{len(s)}")
