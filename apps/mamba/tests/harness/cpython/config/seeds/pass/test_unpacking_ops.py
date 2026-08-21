# Operational AssertionPass seed for sequence unpacking semantics.
# Surface: positional unpack, starred head/tail capture, multi-assign,
# swap via tuple, nested unpack, dict.items() unpack in for-loop.
# Companion to stub/test_unpack.py — vendored unittest seed.
_ledger: list[int] = []

a, b, c = 1, 2, 3
assert (a, b, c) == (1, 2, 3); _ledger.append(1)

a, b = b, a
assert (a, b) == (2, 1); _ledger.append(1)

x, *rest = [1, 2, 3, 4]
assert x == 1; _ledger.append(1)
assert rest == [2, 3, 4]; _ledger.append(1)

*init, last = [1, 2, 3, 4]
assert init == [1, 2, 3]; _ledger.append(1)
assert last == 4; _ledger.append(1)

head, *mid, tail = [1, 2, 3, 4, 5]
assert head == 1; _ledger.append(1)
assert mid == [2, 3, 4]; _ledger.append(1)
assert tail == 5; _ledger.append(1)

d = {"a": 1, "b": 2}
collected: list[tuple] = []
for k, v in d.items():
    collected.append((k, v))
assert sorted(collected) == [("a", 1), ("b", 2)]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_unpacking_ops {sum(_ledger)} asserts")
