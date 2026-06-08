x = 10
y = 3.14159
name = "Alice"

# basic
print(f"x = {x}")
print(f"y = {y}")
print(f"{name} is {x}")

# arithmetic
print(f"{x + 5}")
print(f"{x * 2}")
print(f"{(x + 1) * 2}")

# method calls
print(f"{name.upper()}")
print(f"{name.lower()}")
print(f"{len(name)}")

# container access
items = [1, 2, 3]
print(f"{items[0]}")
print(f"{items[-1]}")
print(f"{items}")

d = {"a": 1, "b": 2}
print(f"{d['a']}")

# conditional
print(f"{x if x > 0 else -x}")

# format spec
print(f"{x:5}")
print(f"{x:05}")
print(f"{y:.2f}")
print(f"{y:.4f}")

# padding
s = "hi"
print(f"{s:>5}|")
print(f"{s:<5}|")
print(f"{s:^5}|")

# hex / oct / bin
print(f"{x:x}")
print(f"{x:o}")
print(f"{x:b}")
