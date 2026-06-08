# Dict literal unpacking: {**d1, **d2, key: val}
defaults = {"x": 1, "y": 2}
overrides = {"y": 10, "z": 30}

merged = {**defaults, **overrides}
print(len(merged))
print(merged["x"])
print(merged["y"])
print(merged["z"])

# Unpack with additional literal keys
combined = {**defaults, "a": 100, "b": 200}
print(len(combined))
print(combined["a"])
print(combined["b"])
print(combined["x"])

# Empty unpack
empty = {}
result = {**empty, "key": 42}
print(len(result))
print(result["key"])

# Override order: later keys win
d1 = {"a": 1, "b": 2}
d2 = {"b": 3, "c": 4}
d3 = {**d1, **d2}
print(d3["a"])
print(d3["b"])
print(d3["c"])
