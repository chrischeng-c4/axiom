# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict deep broad

# update from dict
d = {"a": 1, "b": 2}
d.update({"b": 20, "c": 3})
print(sorted(d.items()))

# update from pairs (iterable) — may or may not work
# d.update([("d", 4), ("e", 5)])   # untested

# pop default
d2 = {"x": 1, "y": 2}
print(d2.pop("x"))
print(sorted(d2.items()))
print(d2.pop("missing", "default"))
print(sorted(d2.items()))

# setdefault
d3 = {"a": 1}
print(d3.setdefault("a", 999))
print(d3.setdefault("b", 10))
print(sorted(d3.items()))

# get with default
d4 = {"k": "v"}
print(d4.get("k"))
print(d4.get("missing"))
print(d4.get("missing", "fallback"))

# in membership
d5 = {"a": 1, "b": 2}
print("a" in d5)
print("c" in d5)
print("c" not in d5)

# nested dict
d6 = {"outer": {"inner": 42}}
print(d6["outer"]["inner"])

# copy
d7 = {"a": 1, "b": 2}
d8 = d7.copy()
d8["c"] = 3
print(sorted(d7.items()))
print(sorted(d8.items()))

# keys/values/items
d9 = {"x": 10, "y": 20, "z": 30}
print(sorted(d9.keys()))
print(sorted(d9.values()))
print(sorted(d9.items()))

# len
print(len({}))
print(len({"a": 1}))
print(len({"a": 1, "b": 2, "c": 3}))

# iterate
d10 = {"a": 1, "b": 2}
for k in sorted(d10):
    print(k, d10[k])

# contains value
d11 = {"a": 1, "b": 2}
print(1 in d11.values())
print(99 in d11.values())
