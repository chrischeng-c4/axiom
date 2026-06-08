# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict iteration order broad (insertion order per PEP 468)

# basic iteration
d = {"c": 3, "a": 1, "b": 2}
for k in d:
    print(k, d[k])

# keys()
d = {"a": 1, "b": 2, "c": 3}
print(list(d.keys()))

# values()
print(list(d.values()))

# items()
print(list(d.items()))

# iteration preserves order
d = {}
d["z"] = 1
d["a"] = 2
d["m"] = 3
for k in d:
    print(k)

# for k, v in items
d = {"x": 10, "y": 20, "z": 30}
for k, v in d.items():
    print(k, "->", v)

# sum of values
d = {"a": 1, "b": 2, "c": 3}
total = sum(d.values())
print(total)

# max by value
mx = max(d.values())
print(mx)

# building dict via update from list of tuples
pairs = [("a", 1), ("b", 2), ("c", 3)]
dd = {}
dd.update(pairs)
print(sorted(dd.items()))

# filter dict
d = {"a": 1, "b": 2, "c": 3, "d": 4}
evens = {k: v for k, v in d.items() if v % 2 == 0}
print(sorted(evens.items()))

# invert dict
inv = {v: k for k, v in d.items()}
print(sorted(inv.items()))

# nested dict access
nested = {"outer": {"inner": 42}}
print(nested["outer"]["inner"])

# dict of lists
grouped = {"evens": [2, 4, 6], "odds": [1, 3, 5]}
print(grouped["evens"])
print(grouped["odds"])
print(len(grouped["evens"]))

# accumulate via setdefault
g = {}
for x in [("a", 1), ("a", 2), ("b", 3), ("a", 4)]:
    g.setdefault(x[0], []).append(x[1])
print(sorted(g.items()))

# dict iteration with sorted keys
d = {"c": 3, "a": 1, "b": 2}
for k in sorted(d.keys()):
    print(k, d[k])
