# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict.popitem() — LIFO order (Python 3.7+ preserves insertion order)

d = {"a": 1, "b": 2, "c": 3}

# Last inserted first
print(d.popitem())
print(d.popitem())
print(d.popitem())

# Popitem on empty raises KeyError
try:
    d.popitem()
except KeyError as e:
    print("raised")

# Mixed insertions
e = {}
e["x"] = 1
e["y"] = 2
e["z"] = 3
print(e.popitem())
print(e.popitem())
print(e.popitem())
