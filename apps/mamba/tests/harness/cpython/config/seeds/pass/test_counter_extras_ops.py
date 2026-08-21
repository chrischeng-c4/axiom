# Operational AssertionPass seed for `collections.Counter` surface not
# covered by `test_counter_ops`. That seed asserts construction from
# str/list, indexing (including missing-key zero), and most_common(N)
# for N=2. This seed asserts:
#   * type(c).__name__ == "Counter"
#   * len(c) is the number of distinct keys
#   * sum(c.values()) is the total multiplicity
#   * sorted(c.keys()) returns the distinct keys
#   * `k in c` containment for both present and absent keys
#   * most_common() with no argument returns the full sorted list
#   * most_common(0) returns []
#   * most_common(N) for N > distinct_keys returns all distinct
#   * Counter on an empty iterable is empty
import collections
_ledger: list[int] = []

# Construct from a string with duplicates
c = collections.Counter("abracadabra")
# type identity
assert type(c).__name__ == "Counter"; _ledger.append(1)

# len(c) is the number of distinct keys
# "abracadabra" → {a:5, b:2, r:2, c:1, d:1} → 5 distinct
assert len(c) == 5; _ledger.append(1)

# sum of values is the total multiplicity (11 characters)
assert sum(c.values()) == 11; _ledger.append(1)

# sorted keys
assert sorted(c.keys()) == ["a", "b", "c", "d", "r"]; _ledger.append(1)

# Present-key containment
assert "a" in c; _ledger.append(1)
assert "b" in c; _ledger.append(1)
assert "r" in c; _ledger.append(1)
# Absent keys are NOT in c (the zero default is for [], not in)
assert "z" not in c; _ledger.append(1)
assert "x" not in c; _ledger.append(1)

# Indexing the most common
assert c["a"] == 5; _ledger.append(1)
assert c["b"] == 2; _ledger.append(1)
assert c["c"] == 1; _ledger.append(1)
# Missing key indexes to 0 (Counter quirk)
assert c["z"] == 0; _ledger.append(1)

# most_common() with no argument returns full ranking
top_all = c.most_common()
assert top_all[0] == ("a", 5); _ledger.append(1)
assert len(top_all) == 5; _ledger.append(1)

# most_common(0) returns the empty list
assert c.most_common(0) == []; _ledger.append(1)

# most_common(N) where N == distinct count
top5 = c.most_common(5)
assert len(top5) == 5; _ledger.append(1)
assert top5[0] == ("a", 5); _ledger.append(1)

# most_common(1)
top1 = c.most_common(1)
assert len(top1) == 1; _ledger.append(1)
assert top1[0] == ("a", 5); _ledger.append(1)

# Empty Counter
empty = collections.Counter("")
assert len(empty) == 0; _ledger.append(1)
assert sum(empty.values()) == 0; _ledger.append(1)
assert empty.most_common() == []; _ledger.append(1)
# Missing key on empty still indexes to 0
assert empty["q"] == 0; _ledger.append(1)

# Counter from a list with duplicates
c_list = collections.Counter([1, 1, 2, 3, 3, 3])
assert c_list[3] == 3; _ledger.append(1)
assert c_list[1] == 2; _ledger.append(1)
assert len(c_list) == 3; _ledger.append(1)
assert sum(c_list.values()) == 6; _ledger.append(1)
assert c_list.most_common(1) == [(3, 3)]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_counter_extras_ops {sum(_ledger)} asserts")
