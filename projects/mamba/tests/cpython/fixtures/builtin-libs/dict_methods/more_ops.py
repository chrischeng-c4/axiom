# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
d = {"a": 1, "b": 2, "c": 3}

print(d["a"])
print(d.get("a"))
print(d.get("x"))
print(d.get("x", 99))
print("a" in d)
print("x" in d)
print(len(d))

print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))

d["d"] = 4
print(sorted(d.items()))

d["a"] = 10
print(d["a"])

v = d.pop("a")
print(v)
print(sorted(d.items()))

v = d.pop("missing", -1)
print(v)

d2 = {"x": 1}
d2.setdefault("x", 99)
print(d2["x"])
d2.setdefault("y", 99)
print(d2["y"])

d3 = {"a": 1}
d3.update({"b": 2, "c": 3})
print(sorted(d3.items()))

d4 = {"a": 1}
d4.update([("b", 2), ("c", 3)])
print(sorted(d4.items()))

d5 = {"a": 1, "b": 2}
d5.clear()
print(d5)
print(len(d5))

d6 = {"x": 1, "y": 2}
c6 = d6.copy()
print(sorted(c6.items()))

print(dict([("a", 1), ("b", 2)]))

# comprehension merge
a = {"x": 1, "y": 2}
b = {"y": 20, "z": 30}
merged = {**a, **b}
print(sorted(merged.items()))

total = 0
for k in {"a": 1, "b": 2, "c": 3}:
    total += 1
print(total)

print(sum({"a": 1, "b": 2, "c": 3}.values()))

# nested dict
n = {"outer": {"inner": {"x": 1, "y": 2}}}
print(n["outer"]["inner"]["x"])
print(sorted(n["outer"]["inner"].items()))

# dict equality
print({"a": 1, "b": 2} == {"b": 2, "a": 1})
print({"a": 1} == {"a": 2})
