# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict broader ops

d = {"a": 1, "b": 2, "c": 3}

# keys / values / items
print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))

# length and membership
print(len(d))
print("a" in d)
print("z" in d)

# get / default
print(d.get("a"))
print(d.get("z"))
print(d.get("z", "default"))

# pop
d3 = {"a": 1, "b": 2}
print(d3.pop("a"))
print(d3.pop("z", "default"))
print(sorted(d3.keys()))

# setdefault
d4 = {"a": 1}
print(d4.setdefault("a", 99))
print(d4.setdefault("b", 99))
print(sorted(d4.items()))

# dict merge / |
d5 = {"x": 1, "y": 2}
d6 = {"y": 20, "z": 30}
merged = d5 | d6
print(sorted(merged.items()))

# clear
d7 = {"a": 1, "b": 2}
d7.clear()
print(len(d7))

# copy (shallow)
d8 = {"a": 1, "b": 2}
d9 = d8.copy()
print(sorted(d9.items()))
