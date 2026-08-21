# test_random.py — #2836 CPython random seed (executed assertions).
#
# Mamba-authored seed distilled from the random module surface.
# Exercises the runtime-portable invariants — every PRNG implementation
# must satisfy these, regardless of the underlying generator's internal
# state machine — so the seed works on BOTH mamba and CPython without
# pinning specific sample values. Per the #2836 acceptance:
# "Fixture avoids platform-specific randomness."
#
# Determinism contract: `seed(N); x = call(); seed(N); y = call()`
# MUST produce `x == y` (the load-bearing property of every PRNG).
# Range contract: every helper respects its documented bounds.
# Set contract: choice/sample return values from the input population.
#
# Mamba's PRNG produces different values from CPython for the same
# seed (different generator algorithm internally), so absolute samples
# can't be compared across runtimes; the determinism + range + set
# invariants ARE testable on both.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: random N asserts` to stdout.

import random

_ledger: list[int] = []

# 1. Module identity + public surface bindings.
assert random.__name__ == "random", "random.__name__ must be 'random'"
_ledger.append(1)
assert hasattr(random, "seed"), "random must expose seed"
_ledger.append(1)
assert hasattr(random, "random"), "random must expose random"
_ledger.append(1)
assert hasattr(random, "randint"), "random must expose randint"
_ledger.append(1)
assert hasattr(random, "choice"), "random must expose choice"
_ledger.append(1)
assert hasattr(random, "shuffle"), "random must expose shuffle"
_ledger.append(1)
assert hasattr(random, "sample"), "random must expose sample"
_ledger.append(1)
assert hasattr(random, "uniform"), "random must expose uniform"
_ledger.append(1)
assert hasattr(random, "randrange"), "random must expose randrange"
_ledger.append(1)

# 2. Determinism — the load-bearing PRNG invariant: seeding to the
#    same value produces the same draw.
random.seed(42)
_r1 = random.random()
random.seed(42)
_r2 = random.random()
assert _r1 == _r2, "seed(42); random() is deterministic across reseed"
_ledger.append(1)

random.seed(123)
_ri1 = random.randint(1, 1000)
random.seed(123)
_ri2 = random.randint(1, 1000)
assert _ri1 == _ri2, "seed(123); randint is deterministic across reseed"
_ledger.append(1)

# 3. random() — must be in [0.0, 1.0).
random.seed(7)
_x = random.random()
assert _x >= 0.0, "random() returns value >= 0.0"
_ledger.append(1)
assert _x < 1.0, "random() returns value < 1.0 (half-open upper bound)"
_ledger.append(1)

# 4. randint(a, b) — inclusive on both ends.
random.seed(7)
_ri = random.randint(5, 10)
assert _ri >= 5, "randint(5, 10) >= 5 (inclusive lower)"
_ledger.append(1)
assert _ri <= 10, "randint(5, 10) <= 10 (inclusive upper)"
_ledger.append(1)
# randint(n, n) — degenerate one-point range; the only legal answer is n.
random.seed(7)
assert random.randint(42, 42) == 42, "randint(42, 42) == 42 (one-point range)"
_ledger.append(1)

# 5. uniform(a, b) — continuous range, inclusive.
random.seed(7)
_u = random.uniform(0.0, 10.0)
assert _u >= 0.0, "uniform(0, 10) >= 0.0"
_ledger.append(1)
assert _u <= 10.0, "uniform(0, 10) <= 10.0"
_ledger.append(1)

# 6. randrange(start, stop) — half-open [start, stop).
random.seed(7)
_rr = random.randrange(0, 100)
assert _rr >= 0, "randrange(0, 100) >= 0"
_ledger.append(1)
assert _rr < 100, "randrange(0, 100) < 100 (half-open upper bound)"
_ledger.append(1)

# 7. choice — must return one of the input elements.
random.seed(7)
_c = random.choice([100, 200, 300])
assert _c == 100 or _c == 200 or _c == 300, "choice([100,200,300]) returns one of the inputs"
_ledger.append(1)

# 8. shuffle — preserves the multiset (len and sorted both unchanged),
#    just reorders.
random.seed(7)
_lst = [1, 2, 3, 4, 5]
random.shuffle(_lst)
assert len(_lst) == 5, "shuffle preserves length"
_ledger.append(1)
assert sorted(_lst) == [1, 2, 3, 4, 5], "shuffle preserves the multiset (sorted is unchanged)"
_ledger.append(1)

# 9. sample(population, k) — returns k distinct elements from
#    population; each must be from the source.
random.seed(7)
_s = random.sample([1, 2, 3, 4, 5], 3)
assert len(_s) == 3, "sample(pop, 3) returns 3 elements"
_ledger.append(1)
assert _s[0] in [1, 2, 3, 4, 5], "sample[0] is from population"
_ledger.append(1)
assert _s[1] in [1, 2, 3, 4, 5], "sample[1] is from population"
_ledger.append(1)
assert _s[2] in [1, 2, 3, 4, 5], "sample[2] is from population"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: random {len(_ledger)} asserts")
