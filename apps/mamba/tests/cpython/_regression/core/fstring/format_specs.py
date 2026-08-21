# string formatting / interpolation (working subset)

x = 42
y = 3.14
name = "Bob"

# f-strings
print(f"{x}")
print(f"{x + 1}")
print(f"{x:5d}")
print(f"{x:05d}")
print(f"{y:.2f}")
print(f"{y:8.3f}")
print(f"{name:>10}")
print(f"{name:<10}|")
print(f"{name:^10}|")
print(f"{name:*^10}")

# hex/oct/bin in fstring
print(f"{255:x}")
print(f"{255:X}")
print(f"{255:o}")
print(f"{255:b}")

# nested fstring expr
items = ["a", "b", "c"]
print(f"len is {len(items)}")
print(f"upper: {name.upper()}")
print(f"list: {items}")
print(f"joined: {' '.join(items)}")

# method calls in f-string
print(f"{name.lower()}")
print(f"{[1, 2, 3][0]}")

# str.format
print("{} {}".format("a", "b"))
print("{1} {0}".format("a", "b"))
print("{name}={val}".format(name="x", val=42))
print("{:>10}".format("hi"))
print("{:.3f}".format(1.5))

# comma separator on int
print(f"{1234567:,}")
print(f"{1234567890:,}")

# negative sign
print(f"{-5:+d}")
