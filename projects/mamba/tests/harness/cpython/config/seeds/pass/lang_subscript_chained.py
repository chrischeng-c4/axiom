# Operational AssertionPass seed for the chained-subscript surface.
# Surface: indexing a container that itself contains containers
# composes left-to-right (`m[i][j]`, `deep[i][j][k]`); read-after-
# read works across list-of-lists, dict-of-dicts, list-of-dicts,
# dict-of-lists, tuple-of-tuples, and list-of-strings (string is
# itself indexable, so `s[0][0]` reads a single character);
# write-through chained subscript (`m[i][j] = v`, `nd[k1][k2] = v`)
# mutates the inner container in place; appending to an inner list
# via `mlist[i].append(x)` mutates the outer list's element; adding
# a brand-new inner key via chained subscript (`nd["a"]["y"] = 50`)
# extends the inner dict. Companion to lang_subscript (which covers
# single-level subscript over the same container kinds).
_ledger: list[int] = []

# List of lists — read in every position
m = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
assert m[0][0] == 1; _ledger.append(1)
assert m[1][1] == 5; _ledger.append(1)
assert m[2][2] == 9; _ledger.append(1)
assert m[0][2] == 3; _ledger.append(1)
assert m[2][0] == 7; _ledger.append(1)

# Nested-list write-through — mutate the inner list via chained LHS
m[0][0] = 100
assert m[0][0] == 100; _ledger.append(1)
m[1][1] = 200
assert m[1][1] == 200; _ledger.append(1)

# Dict of dicts — read across both keys
d = {"a": {"x": 1, "y": 2}, "b": {"x": 3, "y": 4}}
assert d["a"]["x"] == 1; _ledger.append(1)
assert d["a"]["y"] == 2; _ledger.append(1)
assert d["b"]["x"] == 3; _ledger.append(1)
assert d["b"]["y"] == 4; _ledger.append(1)

# List of dicts — common record shape
items = [{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]
assert items[0]["id"] == 1; _ledger.append(1)
assert items[0]["name"] == "a"; _ledger.append(1)
assert items[1]["id"] == 2; _ledger.append(1)
assert items[1]["name"] == "b"; _ledger.append(1)

# Dict of lists — read by key then by index
dl = {"x": [10, 20, 30], "y": [40, 50, 60]}
assert dl["x"][0] == 10; _ledger.append(1)
assert dl["x"][2] == 30; _ledger.append(1)
assert dl["y"][1] == 50; _ledger.append(1)

# Triple nesting — three-level chained subscript
deep = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
assert deep[0][0][0] == 1; _ledger.append(1)
assert deep[1][1][1] == 8; _ledger.append(1)
assert deep[0][1][0] == 3; _ledger.append(1)
assert deep[1][0][1] == 6; _ledger.append(1)

# String subscript via list — `s[i]` is a str, then `[j]` is a char
s = ["hello", "world"]
assert s[0][0] == "h"; _ledger.append(1)
assert s[0][4] == "o"; _ledger.append(1)
assert s[1][0] == "w"; _ledger.append(1)
assert s[1][-1] == "d"; _ledger.append(1)

# Tuple of tuples — chained subscript over immutable containers
t = ((1, 2), (3, 4))
assert t[0][0] == 1; _ledger.append(1)
assert t[1][1] == 4; _ledger.append(1)

# Build a 2x2 matrix entirely via chained-subscript writes
ml = [[0, 0], [0, 0]]
ml[0][0] = 1
ml[0][1] = 2
ml[1][0] = 3
ml[1][1] = 4
assert ml == [[1, 2], [3, 4]]; _ledger.append(1)

# Append-through — `mlist[i].append(x)` mutates the inner list
mlist = [[], [], []]
mlist[0].append(1)
mlist[0].append(2)
mlist[1].append(10)
mlist[2].append(100)
assert mlist == [[1, 2], [10], [100]]; _ledger.append(1)

# Dict-of-dict write-through — overwrite existing inner key
nd = {"a": {"x": 1}}
nd["a"]["x"] = 99
assert nd == {"a": {"x": 99}}; _ledger.append(1)

# Dict-of-dict write-through — add a brand-new inner key
nd["a"]["y"] = 50
assert nd["a"]["y"] == 50; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_subscript_chained {sum(_ledger)} asserts")
