# Operational AssertionPass seed for explicit `protocol=` selection
# across all six pickle wire formats (0..5). Companion to
# `test_pickle_ops` (which covers default-protocol surface) and
# `test_pickle_roundtrip_ops` (which exercises payload shapes).
# Surface: `pickle.dumps(obj, protocol=N)` then `pickle.loads(...)`
# round-trips a dict-with-list payload identically for every N in
# 0..5; `pickle.HIGHEST_PROTOCOL` and `pickle.DEFAULT_PROTOCOL`
# expose ints; `HIGHEST_PROTOCOL >= 5` for Py3.12; round-trip of
# `None` returns the exact singleton; round-trip of `True` compares
# equal; nested lists deep-round-trip through protocol-0 and
# protocol-5 alike.
import pickle
_ledger: list[int] = []

payload = {"a": 1, "b": [2, 3], "c": True}

# Per-protocol round-trip (0..5)
r0 = pickle.loads(pickle.dumps(payload, protocol=0))
assert r0 == payload; _ledger.append(1)

r1 = pickle.loads(pickle.dumps(payload, protocol=1))
assert r1 == payload; _ledger.append(1)

r2 = pickle.loads(pickle.dumps(payload, protocol=2))
assert r2 == payload; _ledger.append(1)

r3 = pickle.loads(pickle.dumps(payload, protocol=3))
assert r3 == payload; _ledger.append(1)

r4 = pickle.loads(pickle.dumps(payload, protocol=4))
assert r4 == payload; _ledger.append(1)

r5 = pickle.loads(pickle.dumps(payload, protocol=5))
assert r5 == payload; _ledger.append(1)

# Protocol constants
assert isinstance(pickle.HIGHEST_PROTOCOL, int); _ledger.append(1)
assert isinstance(pickle.DEFAULT_PROTOCOL, int); _ledger.append(1)
assert pickle.HIGHEST_PROTOCOL >= 5; _ledger.append(1)

# None / True round-trip
assert pickle.loads(pickle.dumps(None)) is None; _ledger.append(1)
assert pickle.loads(pickle.dumps(True)) == True; _ledger.append(1)
assert pickle.loads(pickle.dumps(False)) == False; _ledger.append(1)

# Nested list deep round-trip under protocol-0 and protocol-5
nested = [[1, 2], [3, [4, 5]]]
assert pickle.loads(pickle.dumps(nested, protocol=0)) == nested; _ledger.append(1)
assert pickle.loads(pickle.dumps(nested, protocol=5)) == nested; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pickle_protocol_all_ops {sum(_ledger)} asserts")
