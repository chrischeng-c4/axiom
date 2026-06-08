# RUN: parse
# CPython 3.12 test_dict: dict operations

# Construction
d = {}
d = {"a": 1, "b": 2}
d = dict()
d = dict(a=1, b=2)
d = dict([("a", 1), ("b", 2)])
d = dict({"a": 1})

# Access
d = {"key": "value"}
v = d["key"]
v = d.get("key")
v = d.get("missing", None)

# Modification
d["new_key"] = "new_value"
del d["key"]
d.update({"c": 3})
d.update(c=3, d=4)
v = d.pop("key")
v = d.pop("key", None)
v = d.setdefault("key", "default")
d.clear()

# Views
d = {"a": 1, "b": 2, "c": 3}
keys = d.keys()
vals = d.values()
items = d.items()

# Membership
b = "a" in d
b = "z" not in d

# Iteration
for k in d:
    pass
for k, v in d.items():
    pass
for v in d.values():
    pass

# Dict comprehension
squares = {x: x**2 for x in range(5)}
filtered = {k: v for k, v in d.items() if v > 0}

# Nested dicts
nested = {"outer": {"inner": 42}}
v = nested["outer"]["inner"]

# len
n = len(d)

# copy
d2 = d.copy()
