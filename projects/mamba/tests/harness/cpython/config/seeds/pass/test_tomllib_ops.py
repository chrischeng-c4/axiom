# Operational AssertionPass seed for `tomllib.loads` (read-only TOML
# parser introduced in CPython 3.11).
# Surface: loads(str) returns a dict; primitive scalars (str, int,
# float, bool) round-trip; arrays parse as list; nested [section]
# tables parse as nested dicts; empty input returns the empty dict.
import tomllib
_ledger: list[int] = []
d = tomllib.loads('key = "value"\nn = 42\nf = 3.14\nb = true\narr = [1, 2, 3]\n[section]\nname = "foo"')
# loads returns a dict
assert isinstance(d, dict); _ledger.append(1)
# Primitive scalars round-trip
assert d["key"] == "value"; _ledger.append(1)
assert d["n"] == 42; _ledger.append(1)
assert d["f"] == 3.14; _ledger.append(1)
assert d["b"] == True; _ledger.append(1)
# Inline arrays parse as a Python list
assert d["arr"] == [1, 2, 3]; _ledger.append(1)
# [section] header projects to a nested dict
assert isinstance(d["section"], dict); _ledger.append(1)
assert d["section"]["name"] == "foo"; _ledger.append(1)
# Empty TOML is the empty dict
empty = tomllib.loads("")
assert empty == {}; _ledger.append(1)
assert len(empty) == 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_tomllib_ops {sum(_ledger)} asserts")
