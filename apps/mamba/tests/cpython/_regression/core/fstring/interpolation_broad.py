# f-string interpolation patterns broad

# simple var
name = "alice"
print(f"{name}")
print(f"hi {name}")

# int
n = 42
print(f"{n}")
print(f"n={n}")

# float
pi = 3.14159
print(f"{pi}")

# multiple
first = "alice"
last = "smith"
print(f"{first} {last}")
print(f"{first}-{last}")

# expression
x = 10
y = 20
print(f"{x + y}")
print(f"{x * y}")
print(f"{x - y}")

# method call
s = "hello"
print(f"{s.upper()}")
print(f"{s[0]}")
print(f"{len(s)}")

# repeated var
print(f"{x} and {x} and {x}")

# format spec :d
print(f"{n:d}")
print(f"{n:5d}")
print(f"{n:05d}")

# format spec :f
print(f"{pi:.2f}")
print(f"{pi:.4f}")
print(f"{pi:.0f}")

# format spec string align
print(f"{name:>10}")
print(f"{name:<10}|")
print(f"{name:^10}")

# format spec with fill
print(f"{name:*<10}")
print(f"{name:*>10}")

# conditional inline
score = 95
print(f"{'pass' if score >= 60 else 'fail'}")

# with list/dict access
data = [10, 20, 30]
print(f"{data[0]}")
print(f"{data[-1]}")
print(f"{data[0] + data[1]}")

# escape curly
print(f"{{literal}}")
print(f"{{a}} and {name}")

# math in expression
print(f"sum={1 + 2 + 3}")
print(f"pow={2 ** 10}")

# bool
flag = True
print(f"{flag}")
print(f"{not flag}")

# comparison
print(f"{x < y}")
print(f"{x > y}")
print(f"{x == y}")

# format int as hex/oct/bin
print(f"{255:x}")
print(f"{255:X}")
print(f"{255:o}")
print(f"{255:b}")
