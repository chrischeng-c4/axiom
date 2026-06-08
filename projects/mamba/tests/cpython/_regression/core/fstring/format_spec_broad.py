# fstring format spec broad

# integer formatting
n = 42
print(f"{n}")
print(f"{n:d}")
print(f"{n:5d}")
print(f"{n:05d}")

# hex/oct/bin
print(f"{255:x}")
print(f"{255:X}")
print(f"{255:o}")
print(f"{255:b}")

# float formatting
x = 3.14159
print(f"{x}")
print(f"{x:.2f}")
print(f"{x:.4f}")
print(f"{x:.0f}")
print(f"{x:10.2f}")

# strings - width/align
s = "hi"
print(f"[{s:10}]")
print(f"[{s:<10}]")
print(f"[{s:>10}]")
print(f"[{s:^10}]")
print(f"[{s:*^10}]")

# expressions inside
a = 3
b = 4
print(f"{a + b}")
print(f"{a * b}")
print(f"{a * 2 + b}")

# nested attribute/index
lst = [10, 20, 30]
print(f"{lst[0]}")
print(f"{lst[1]}")
print(f"{lst[2]}")

d = {"x": 100}
print(f"{d['x']}")

# conversion: !s
print(f"{s!s}")

# multiple in one string
name = "alice"
age = 30
print(f"{name} is {age}")
print(f"[{name:<10}|{age:>5}]")

# nested fstring
print(f"{f'{1+1}'}")
print(f"[{f'{name}':^15}]")
