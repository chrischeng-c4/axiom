# Operational AssertionPass seed for collections.deque, Counter,
# defaultdict, and ChainMap operational surfaces.
# Surface:
#   • deque: append, appendleft, pop, popleft, extend, extendleft,
#     rotate (positive / negative offset), clear, maxlen bound
#     evicting from the opposite end on overflow;
#   • Counter: dict-style read, most_common(n), addition merging
#     element counts across two Counters, subtraction zeroing
#     negative carries;
#   • defaultdict(list): bucket-append idiom; missing-key creates
#     an empty list automatically;
#   • ChainMap: layered lookup picks the leftmost dict that has the
#     key; missing in earlier layer falls through to a later one.
#
# `len(deque)`, `deque.maxlen` attribute, Counter subtraction, and
# `OrderedDict.move_to_end` are deliberately NOT exercised here —
# mamba 0.3.60 returns 0 / None / empty-Counter / AttributeError on
# these respectively; each moves to spec/ as a focused growth seed.
from collections import deque, Counter, defaultdict, ChainMap
_ledger: list[int] = []

# deque init + list materialisation
d = deque([1, 2, 3])
assert list(d) == [1, 2, 3]; _ledger.append(1)

# append / appendleft
d.append(4)
assert list(d) == [1, 2, 3, 4]; _ledger.append(1)
d.appendleft(0)
assert list(d) == [0, 1, 2, 3, 4]; _ledger.append(1)

# pop / popleft return values + after-state
right = d.pop()
assert right == 4; _ledger.append(1)
assert list(d) == [0, 1, 2, 3]; _ledger.append(1)
left = d.popleft()
assert left == 0; _ledger.append(1)
assert list(d) == [1, 2, 3]; _ledger.append(1)

# extend appends each item; extendleft reverses-then-prepends
d.extend([10, 20])
assert list(d) == [1, 2, 3, 10, 20]; _ledger.append(1)
d.extendleft([99, 98])
assert list(d) == [98, 99, 1, 2, 3, 10, 20]; _ledger.append(1)

# rotate(n) shifts right by n; rotate(-n) shifts left
d.rotate(1)
assert list(d) == [20, 98, 99, 1, 2, 3, 10]; _ledger.append(1)
d.rotate(-2)
assert list(d) == [99, 1, 2, 3, 10, 20, 98]; _ledger.append(1)

# clear() empties the deque
d.clear()
assert list(d) == []; _ledger.append(1)

# maxlen bound — pushing past maxlen drops from the opposite end
b = deque([], maxlen=3)
for i in range(5):
    b.append(i)
assert list(b) == [2, 3, 4]; _ledger.append(1)

# Counter — dict-style counting
c = Counter("aabbbcccc")
assert c["a"] == 2; _ledger.append(1)
assert c["b"] == 3; _ledger.append(1)
assert c["c"] == 4; _ledger.append(1)
assert c["z"] == 0; _ledger.append(1)  # missing keys return 0

# most_common(n) — top-n by count
mc = c.most_common(2)
assert mc == [("c", 4), ("b", 3)]; _ledger.append(1)

# Counter arithmetic — addition merges counts
c2 = c + c
assert c2["a"] == 4; _ledger.append(1)
assert c2["b"] == 6; _ledger.append(1)
assert c2["c"] == 8; _ledger.append(1)

# defaultdict(list) — bucket-append idiom
dd = defaultdict(list)
dd["k"].append(1)
dd["k"].append(2)
dd["m"].append(3)
assert dd["k"] == [1, 2]; _ledger.append(1)
assert dd["m"] == [3]; _ledger.append(1)
assert dict(dd) == {"k": [1, 2], "m": [3]}; _ledger.append(1)

# defaultdict(int) — counter-like accumulator
dd2 = defaultdict(int)
for ch in "abca":
    dd2[ch] += 1
assert dd2["a"] == 2; _ledger.append(1)
assert dd2["b"] == 1; _ledger.append(1)
assert dd2["c"] == 1; _ledger.append(1)

# ChainMap — leftmost-layer-wins lookup
cm = ChainMap({"a": 1}, {"a": 99, "b": 2})
assert cm["a"] == 1; _ledger.append(1)
assert cm["b"] == 2; _ledger.append(1)

# ChainMap fallthrough on missing key in leftmost layer
cm2 = ChainMap({}, {"only_in_back": 42})
assert cm2["only_in_back"] == 42; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_collections_deque_ops {sum(_ledger)} asserts")
