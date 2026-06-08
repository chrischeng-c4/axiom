# F-string conversion specifiers !r, !s, !a (PEP 498).
# !r => repr(); !s => str() (default); !a => ascii()-equivalent (repr for mamba).

s = "hello"
# Default: str() — no quotes.
print(f"{s}")
print(f"[{s}]")

# !r — repr() — quotes the string.
print(f"{s!r}")
print(f"[{s!r}]")

# !s — explicit str(), same as default.
print(f"{s!s}")
print(f"[{s!s}]")

# !a — ascii()-equivalent (quoted like repr).
print(f"{s!a}")

# Non-string values — str == repr for ints / bools / None, but list[str] differs.
n = 42
print(f"{n!r}")
print(f"{n!s}")

lst = ["a", "b"]
print(f"{lst!s}")
print(f"{lst!r}")

# Conversion + format spec.
print(f"{s!r:>10}")
print(f"{s!r:.2}")

# `!=` should NOT be treated as a conversion.
x, y = 1, 2
print(f"{x != y}")
print(f"{x == y}")

# `!r` inside string / dict-key must not fool the outer parser.
d = {"a!r": "val"}
print(f"{d['a!r']}")

# Nested expressions with conversion.
print(f"sum={(1 + 2)!r}")

# Multiple fields with conversion.
a, b = "x", "y"
print(f"{a!r} and {b!r}")
