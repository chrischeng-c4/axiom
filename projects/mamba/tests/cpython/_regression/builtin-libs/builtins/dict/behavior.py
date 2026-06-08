"""Behavior contract for builtins.dict.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: dict() with no args returns {}
assert dict() == {}, f"dict() = {dict()!r}"

# Rule 2: dict(mapping) — from another dict
d = dict({"a": 1, "b": 2})
assert d == {"a": 1, "b": 2}, f"dict(mapping) = {d!r}"

# Rule 3: dict(kwargs)
d = dict(x=1, y=2)
assert d == {"x": 1, "y": 2}, f"dict(kwargs) = {d!r}"

# Rule 4: dict(iterable of pairs)
d = dict([("a", 1), ("b", 2)])
assert d == {"a": 1, "b": 2}, f"dict(pairs) = {d!r}"

# Rule 5: get — present and missing
d = {"a": 1}
assert d.get("a") == 1, f"get present = {d.get('a')!r}"
assert d.get("b") is None, f"get missing = {d.get('b')!r}"
assert d.get("b", 99) == 99, f"get default = {d.get('b', 99)!r}"

# Rule 6: setdefault
d = {"a": 1}
assert d.setdefault("a", 99) == 1, "setdefault existing changed"
assert d.setdefault("b", 2) == 2, "setdefault new wrong"
assert d["b"] == 2, "setdefault new not in dict"

# Rule 7: update from dict
d = {"a": 1}
d.update({"b": 2, "a": 99})
assert d == {"a": 99, "b": 2}, f"update = {d!r}"

# Rule 8: pop
d = {"a": 1, "b": 2}
val = d.pop("a")
assert val == 1, f"pop val = {val!r}"
assert "a" not in d, "pop didn't remove key"
assert d.pop("z", 0) == 0, "pop missing with default"

# Rule 9: pop missing raises KeyError
_raised = False
try:
    {"a": 1}.pop("b")
except KeyError:
    _raised = True
assert _raised, "pop missing did not raise KeyError"

# Rule 10: keys / values / items
d = {"a": 1, "b": 2}
assert list(d.keys()) == ["a", "b"], f"keys = {list(d.keys())!r}"
assert list(d.values()) == [1, 2], f"values = {list(d.values())!r}"
assert list(d.items()) == [("a", 1), ("b", 2)], f"items = {list(d.items())!r}"

# Rule 11: in operator
d = {"a": 1}
assert "a" in d, "'a' in d failed"
assert "b" not in d, "'b' not in d failed"

# Rule 12: len
d = {"a": 1, "b": 2, "c": 3}
assert len(d) == 3, f"len(d) = {len(d)!r}"

# Rule 13: del key
d = {"a": 1, "b": 2}
del d["a"]
assert d == {"b": 2}, f"after del: {d!r}"

# Rule 14: clear
d = {"a": 1}
d.clear()
assert d == {}, f"clear = {d!r}"

# Rule 15: copy is shallow
d = {"a": [1, 2]}
cp = d.copy()
cp["a"].append(3)
assert d["a"] == [1, 2, 3], "copy is shallow"

# Rule 16: dict insertion order (Python 3.7+)
d = {}
for k in ["c", "a", "b"]:
    d[k] = 1
assert list(d.keys()) == ["c", "a", "b"], f"insertion order = {list(d.keys())!r}"

# Rule 17: fromkeys
d = dict.fromkeys(["a", "b", "c"], 0)
assert d == {"a": 0, "b": 0, "c": 0}, f"fromkeys = {d!r}"

print("behavior OK")
