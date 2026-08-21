# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict methods — broad coverage

# get with/without default
d = {"a": 1, "b": 2}
print(d.get("a"))
print(d.get("c"))
print(d.get("c", 99))
print(d.get("a", 99))

# setdefault
d = {}
print(d.setdefault("x", 10))
print(d.setdefault("x", 20))  # existing key unchanged
print(sorted(d.items()))

# update from dict
d = {"a": 1}
d.update({"b": 2, "c": 3})
print(sorted(d.items()))

# update from list of pairs
d = {"a": 1}
d.update([("b", 2), ("c", 3)])
print(sorted(d.items()))

# update from tuple of pairs
d = {"a": 1}
d.update((("b", 2), ("c", 3)))
print(sorted(d.items()))

# pop with/without default
d = {"a": 1, "b": 2}
print(d.pop("a"))
print(d.pop("c", 99))
print(sorted(d.items()))

# keys / values / items
d = {"x": 10, "y": 20}
print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))

# membership
print("x" in d)
print("z" in d)

# copy
d2 = d.copy()
d2["new"] = 99
print("new" in d)
print("new" in d2)

# len
print(len(d))
print(len({}))

# iteration order (insertion)
d = {}
d["first"] = 1
d["second"] = 2
d["third"] = 3
for k in d:
    print(k)

# del
d = {"a": 1, "b": 2, "c": 3}
del d["b"]
print(sorted(d.items()))

# clear
d.clear()
print(d)
print(len(d))
