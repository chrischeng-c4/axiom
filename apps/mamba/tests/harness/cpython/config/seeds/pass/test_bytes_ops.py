# Operational AssertionPass seed for builtin `bytes`.
# Surface: literal, len, indexing (returns int), slicing, concat,
# repeat, in/not in, bytes(iterable) constructor.
# Companion to stub/test_bytes.py — vendored unittest seed.
_ledger: list[int] = []
b = b"hello"
assert len(b) == 5; _ledger.append(1)
assert b[0] == 104; _ledger.append(1)
assert b[1] == 101; _ledger.append(1)
assert b[1:3] == b"el"; _ledger.append(1)
assert b[:3] == b"hel"; _ledger.append(1)
assert b[2:] == b"llo"; _ledger.append(1)
assert b + b" world" == b"hello world"; _ledger.append(1)
assert b"a" * 3 == b"aaa"; _ledger.append(1)
assert b"el" in b; _ledger.append(1)
assert b"xy" not in b; _ledger.append(1)
assert bytes([65, 66, 67]) == b"ABC"; _ledger.append(1)
assert len(bytes(5)) == 5; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_bytes_ops {sum(_ledger)} asserts")
