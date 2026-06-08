# Operational AssertionPass seed for the `functools` stdlib module.
# Surface: reduce(no-init), reduce(with init), partial(positional bind).
# Companion to stub/test_functools.py — vendored unittest seed.
from functools import reduce, partial
_ledger: list[int] = []
assert reduce(lambda a, b: a + b, [1, 2, 3, 4, 5]) == 15; _ledger.append(1)
assert reduce(lambda a, b: a * b, [1, 2, 3, 4], 1) == 24; _ledger.append(1)
assert reduce(lambda a, b: a + b, [10]) == 10; _ledger.append(1)
add = lambda a, b: a + b
inc = partial(add, 1)
assert inc(10) == 11; _ledger.append(1)
assert inc(99) == 100; _ledger.append(1)
mul3 = partial(lambda a, b, c: a * b * c, 2, 3)
assert mul3(4) == 24; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_functools_ops {sum(_ledger)} asserts")
