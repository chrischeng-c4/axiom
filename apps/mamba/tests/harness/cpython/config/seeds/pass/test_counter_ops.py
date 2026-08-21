# Operational AssertionPass seed for `collections.Counter`.
# Surface: Counter from iterable, indexing, most_common, missing key
# returns 0.
# Companion to stub/test_counter.py — vendored unittest seed.
from collections import Counter
_ledger: list[int] = []
c = Counter("hello world")
assert c["l"] == 3; _ledger.append(1)
assert c["o"] == 2; _ledger.append(1)
assert c[" "] == 1; _ledger.append(1)
assert c["z"] == 0; _ledger.append(1)
top2 = c.most_common(2)
assert top2[0] == ("l", 3); _ledger.append(1)
assert top2[1] == ("o", 2); _ledger.append(1)
c2 = Counter([1, 1, 2, 3, 3, 3])
assert c2[3] == 3; _ledger.append(1)
assert c2[1] == 2; _ledger.append(1)
assert c2[99] == 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_counter_ops {sum(_ledger)} asserts")
