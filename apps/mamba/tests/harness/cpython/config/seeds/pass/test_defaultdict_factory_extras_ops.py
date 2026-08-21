# Operational AssertionPass seed for `collections.defaultdict` with
# factories not covered by `test_defaultdict_ops` (which exercises
# `int` and `list` factories). This seed asserts:
#   * `str` factory — missing key auto-fills to "";
#   * `float` factory — missing key auto-fills to 0.0;
#   * lambda factory — missing key auto-fills to the lambda's return;
#   * Group-by idiom with list factory — append per key produces
#     bucketed lists (already covered by `test_defaultdict_ops` but
#     this seed asserts the multi-key case across distinct buckets);
#   * Counter-build idiom with int factory — `d[ch] += 1` for ch in
#     a string yields per-character counts.
from collections import defaultdict
_ledger: list[int] = []

# str factory — missing key materializes ""
dstr = defaultdict(str)
assert dstr["empty"] == ""; _ledger.append(1)
# After the read, the key is now in the dict
assert "empty" in dstr; _ledger.append(1)
# Concatenation works on the empty default
dstr["msg"] = "hello"
dstr["msg"] += " world"
assert dstr["msg"] == "hello world"; _ledger.append(1)

# float factory — missing key materializes 0.0
df = defaultdict(float)
assert df["x"] == 0.0; _ledger.append(1)
assert isinstance(df["x"], float); _ledger.append(1)
df["y"] += 1.5
df["y"] += 2.5
assert df["y"] == 4.0; _ledger.append(1)
df["z"] = 3.14
assert df["z"] == 3.14; _ledger.append(1)

# Lambda factory — missing key materializes lambda's return
dl = defaultdict(lambda: 99)
assert dl["missing"] == 99; _ledger.append(1)
dl["explicit"] = 5
assert dl["explicit"] == 5; _ledger.append(1)
# Other missing keys still get the default
assert dl["another"] == 99; _ledger.append(1)

# Lambda factory returning a different default
dl2 = defaultdict(lambda: -1)
assert dl2["foo"] == -1; _ledger.append(1)
assert dl2["bar"] == -1; _ledger.append(1)

# Counter idiom with int factory — count each character
ic = defaultdict(int)
for ch in "abracadabra":
    ic[ch] += 1
assert ic["a"] == 5; _ledger.append(1)
assert ic["b"] == 2; _ledger.append(1)
assert ic["r"] == 2; _ledger.append(1)
assert ic["c"] == 1; _ledger.append(1)
assert ic["d"] == 1; _ledger.append(1)
assert len(ic) == 5; _ledger.append(1)

# Group-by idiom with list factory — bucket items by category
group = defaultdict(list)
records = [("alice", 90), ("bob", 80), ("alice", 95), ("carol", 85), ("bob", 70)]
for name, score in records:
    group[name].append(score)
assert group["alice"] == [90, 95]; _ledger.append(1)
assert group["bob"] == [80, 70]; _ledger.append(1)
assert group["carol"] == [85]; _ledger.append(1)
assert len(group) == 3; _ledger.append(1)
assert sorted(group.keys()) == ["alice", "bob", "carol"]; _ledger.append(1)

# Standard dict ops apply on top of any defaultdict
assert "alice" in group; _ledger.append(1)
assert "missing" not in group; _ledger.append(1)
# defaultdict still acts dict-like for materialised entries
materialized_keys = sorted(group.keys())
assert materialized_keys[0] == "alice"; _ledger.append(1)
assert materialized_keys[-1] == "carol"; _ledger.append(1)

# Mixed read/write — read default 0, then assign, then read assigned
dmix = defaultdict(int)
assert dmix["k"] == 0; _ledger.append(1)
dmix["k"] = 42
assert dmix["k"] == 42; _ledger.append(1)
# A second missing key still gets the default
assert dmix["other"] == 0; _ledger.append(1)
assert len(dmix) == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_defaultdict_factory_extras_ops {sum(_ledger)} asserts")
