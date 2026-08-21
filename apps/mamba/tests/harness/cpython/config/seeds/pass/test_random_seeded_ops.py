# Operational AssertionPass seed for `random` determinism contract.
# Surface: random.seed makes random()/randint/choice/uniform fully
# replayable; same seed → same sequence. Asserts surface invariants
# (deterministic re-seed equality + bounds) rather than CPython-exact
# vectors, because mamba's PRNG implementation is its own and exact
# vector parity with CPython isn't a current goal.
# Companion to stub/test_random.py — vendored unittest seed.
import random
_ledger: list[int] = []
# random() — deterministic under re-seed
random.seed(42)
a = random.random()
random.seed(42)
b = random.random()
assert a == b; _ledger.append(1)
assert 0.0 <= a < 1.0; _ledger.append(1)
# randint(low, high) — bounded inclusively, deterministic under re-seed
random.seed(42)
i1 = random.randint(1, 10)
random.seed(42)
i2 = random.randint(1, 10)
assert i1 == i2; _ledger.append(1)
assert 1 <= i1 <= 10; _ledger.append(1)
# choice — element drawn from the input, deterministic under re-seed
pool = [10, 20, 30, 40, 50]
random.seed(42)
c1 = random.choice(pool)
random.seed(42)
c2 = random.choice(pool)
assert c1 == c2; _ledger.append(1)
assert c1 in pool; _ledger.append(1)
# uniform — bounded, deterministic under re-seed
random.seed(42)
u1 = random.uniform(0.0, 1.0)
random.seed(42)
u2 = random.uniform(0.0, 1.0)
assert u1 == u2; _ledger.append(1)
assert 0.0 <= u1 <= 1.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_random_seeded_ops {sum(_ledger)} asserts")
