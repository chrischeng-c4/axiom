# Operational AssertionPass seed for builtin `dict`.
# Surface: literal/constructor, get with default, in/not in,
# keys/values/items, update, pop, setdefault, len, comprehension,
# iteration order (insertion), equality, copy semantics.
# Companion to stub/test_dict.py — vendored unittest seed.
_ledger: list[int] = []
d = {"a": 1, "b": 2, "c": 3}
assert d["a"] == 1; _ledger.append(1)
assert d.get("a") == 1; _ledger.append(1)
assert d.get("missing") is None; _ledger.append(1)
assert d.get("missing", 99) == 99; _ledger.append(1)
assert "a" in d; _ledger.append(1)
assert "missing" not in d; _ledger.append(1)
assert len(d) == 3; _ledger.append(1)
assert sorted(d.keys()) == ["a", "b", "c"]; _ledger.append(1)
assert sorted(d.values()) == [1, 2, 3]; _ledger.append(1)
d["d"] = 4
assert d["d"] == 4; _ledger.append(1)
assert len(d) == 4; _ledger.append(1)
d.update({"e": 5, "a": 100})
assert d["a"] == 100; _ledger.append(1)
assert d["e"] == 5; _ledger.append(1)
v = d.pop("e")
assert v == 5; _ledger.append(1)
assert "e" not in d; _ledger.append(1)
d.setdefault("f", 42)
assert d["f"] == 42; _ledger.append(1)
d.setdefault("f", 99)
assert d["f"] == 42; _ledger.append(1)
sq = {n: n * n for n in [1, 2, 3, 4]}
assert sq[3] == 9; _ledger.append(1)
assert sq[4] == 16; _ledger.append(1)
e = {"x": 1, "y": 2}
f = {"x": 1, "y": 2}
assert e == f; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_dict_ops {sum(_ledger)} asserts")
