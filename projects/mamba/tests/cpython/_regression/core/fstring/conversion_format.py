# f-string conversion / format-spec — #2809.
#
# Covers f-string expression behavior, conversions (`!r`, `!s`, `!a`),
# simple format specs (width / alignment / precision / zero-pad / hex /
# binary), and nested expressions. Asserts exact rendered strings so
# failure surfaces precisely.
#
# Failure messages name [fstring] so runner stderr identifies the area.


class Item:
    def __init__(self, name):
        self.name = name
    def __repr__(self):
        return "Item(" + self.name + ")"
    def __str__(self):
        return "<" + self.name + ">"


it = Item("widget")

# 1. Conversions: !r (repr), !s (str).
print(f"r={it!r} s={it!s}", "[fstring: conv]")

# 2. !a (ascii) on a plain ascii string is just repr-like.
print(f"a={'abc'!a}", "[fstring: conv-ascii]")

# 3. Width + right alignment (default for ints).
print(f"[{42:5}]", "[fstring: width]")

# 4. Width + left alignment.
print(f"[{42:<5}]", "[fstring: align-left]")

# 5. Width + zero-pad.
print(f"[{42:05}]", "[fstring: zero-pad]")

# 6. Float precision.
print(f"pi~{3.14159:.2f}", "[fstring: precision]")

# 7. Hex / binary integer format specs.
print(f"hex={255:x} HEX={255:X} bin={5:b}", "[fstring: radix]")

# 8. Nested expression — arithmetic inside the braces.
a = 3
b = 4
print(f"sum={a + b} prod={a * b}", "[fstring: expr]")

# 9. Mixed: conversion + format spec is not supported via `!r:spec` here;
#    we use two separate fields.
print(f"name={it.name!s} upper={it.name.upper()!r}", "[fstring: conv-method]")
