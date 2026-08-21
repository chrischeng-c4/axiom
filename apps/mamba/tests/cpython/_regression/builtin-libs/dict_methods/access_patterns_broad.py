# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict access patterns broad

d = {"a": 1, "b": 2, "c": 3, "d": 4}

# basic access
print(d["a"])
print(d["b"])
print(d["c"])
print(d["d"])

# len
print(len(d))
print(len({}))
print(len({"x": 1}))

# in / not in
print("a" in d)
print("z" in d)
print("z" not in d)
print("a" not in d)

# .get with default
print(d.get("a"))
print(d.get("missing"))
print(d.get("missing", 0))
print(d.get("missing", "default"))
print(d.get("a", 999))

# keys/values/items
print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))

# iterate
total = 0
for k in d:
    total += d[k]
print(total)

# iterate values
total = 0
for v in d.values():
    total += v
print(total)

# iterate items
total = 0
for k, v in d.items():
    total += v
print(total)

# mutate
d2 = {"a": 1}
d2["b"] = 2
d2["c"] = 3
d2["a"] = 10  # update
print(sorted(d2.items()))

# del
d3 = {"a": 1, "b": 2, "c": 3}
del d3["b"]
print(sorted(d3.items()))

# pop
d4 = {"a": 1, "b": 2, "c": 3}
v = d4.pop("b")
print(v)
print(sorted(d4.items()))

# pop with default
d5 = {"a": 1}
print(d5.pop("missing", "default"))
print(d5.pop("a", "default"))

# setdefault
d6 = {}
print(d6.setdefault("a", 1))
print(d6.setdefault("a", 99))  # existing, keeps
print(d6.setdefault("b", 2))
print(sorted(d6.items()))

# update
d7 = {"a": 1, "b": 2}
d7.update({"b": 20, "c": 30})
print(sorted(d7.items()))

# empty dict
empty = {}
print(empty)
print(bool(empty))

# dict from list of pairs via loop
pairs = [("k1", 10), ("k2", 20), ("k3", 30)]
d8 = {}
for k, v in pairs:
    d8[k] = v
print(sorted(d8.items()))

# copy
orig = {"a": 1, "b": 2}
c = orig.copy()
c["c"] = 3
print(sorted(orig.items()))
print(sorted(c.items()))

# clear
d9 = {"a": 1, "b": 2}
d9.clear()
print(d9)
print(len(d9))

# values-based
d10 = {"a": 10, "b": 20, "c": 30}
print(sum(d10.values()))
print(max(d10.values()))
print(min(d10.values()))
