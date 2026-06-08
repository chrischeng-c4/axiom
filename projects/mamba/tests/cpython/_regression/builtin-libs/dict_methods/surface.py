# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""dict_methods: surface probes for the dict API (CPython 3.12 oracle)."""

# Every documented dict instance method is present and callable.
for name in (
    "get", "setdefault", "pop", "popitem", "update", "clear",
    "copy", "keys", "values", "items", "fromkeys",
    "__getitem__", "__setitem__", "__delitem__", "__contains__",
    "__len__", "__iter__", "__reversed__", "__or__", "__ior__", "__eq__",
):
    assert hasattr(dict, name), name
    assert callable(getattr(dict, name)), name

# fromkeys is a classmethod usable from the type.
assert dict.fromkeys("ab") == {"a": None, "b": None}

# The three view objects expose their distinguishing members.
d = {"x": 1}
assert hasattr(d.keys(), "isdisjoint")
assert hasattr(d.items(), "isdisjoint")
assert hasattr(d.keys(), "mapping")
assert hasattr(d.values(), "mapping")
assert hasattr(d.items(), "mapping")

# View object type names are stable.
assert type(d.keys()).__name__ == "dict_keys"
assert type(d.values()).__name__ == "dict_values"
assert type(d.items()).__name__ == "dict_items"

print("surface OK")
