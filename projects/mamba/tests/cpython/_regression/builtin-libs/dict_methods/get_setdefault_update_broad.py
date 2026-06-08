# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict get/setdefault/update broad

# get basic
d = {"a": 1, "b": 2}
print(d.get("a"))
print(d.get("b"))
print(d.get("c"))

# get with default
print(d.get("c", -1))
print(d.get("a", -1))
print(d.get("missing", "default"))
print(d.get("missing", 0))

# get doesn't modify
print(sorted(d.items()))

# setdefault basic
d2 = {"a": 1}
print(d2.setdefault("a", 99))  # returns existing
print(d2.setdefault("b", 2))  # sets b=2, returns 2
print(sorted(d2.items()))

d3 = {}
print(d3.setdefault("count", 0))  # sets count=0
print(sorted(d3.items()))

# setdefault + accumulate
counts = {}
for c in "banana":
    counts.setdefault(c, 0)
    counts[c] += 1
print(sorted(counts.items()))

# update with dict
d4 = {"a": 1}
d4.update({"b": 2, "c": 3})
print(sorted(d4.items()))

# update overwrites
d5 = {"a": 1, "b": 2}
d5.update({"a": 99})
print(sorted(d5.items()))

# update with list of tuples
d7 = {}
d7.update([("a", 1), ("b", 2)])
print(sorted(d7.items()))

# pop
d8 = {"a": 1, "b": 2}
print(d8.pop("a"))
print(sorted(d8.items()))

# keys/values/items
d9 = {"a": 1, "b": 2, "c": 3}
print(sorted(d9.keys()))
print(sorted(d9.values()))
print(sorted(d9.items()))

# len/contains
d10 = {"a": 1, "b": 2, "c": 3}
print(len(d10))
print("a" in d10)
print("z" in d10)
print("z" not in d10)

# clear
d11 = {"a": 1, "b": 2}
d11.clear()
print(d11)
print(len(d11))


