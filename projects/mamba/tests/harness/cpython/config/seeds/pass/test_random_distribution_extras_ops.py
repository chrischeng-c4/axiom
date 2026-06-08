# Operational AssertionPass seed for `random.triangular`,
# `random.expovariate`, `random.getrandbits`, and seeded
# reproducibility across `random.random` and `random.uniform`.
# Surface: triangular(0, 1), expovariate(1), gauss(0, 1) each return
# a float; getrandbits(k) returns an int in `[0, 2**k)`. Seed
# determinism: two calls to `random()` after the same `seed(n)`
# yield identical values; the same holds for `uniform`. Within a
# single seeded stream, consecutive `random()` draws differ.
import random
_ledger: list[int] = []

# Distribution functions return floats
random.seed(1)
g = random.gauss(0, 1)
assert isinstance(g, float); _ledger.append(1)

tr = random.triangular(0, 1)
assert isinstance(tr, float); _ledger.append(1)

ev = random.expovariate(1)
assert isinstance(ev, float); _ledger.append(1)

# getrandbits — int in [0, 2**k)
rb = random.getrandbits(8)
assert isinstance(rb, int); _ledger.append(1)
assert 0 <= rb < 256; _ledger.append(1)

rb16 = random.getrandbits(16)
assert 0 <= rb16 < 65536; _ledger.append(1)

rb1 = random.getrandbits(1)
assert rb1 in (0, 1); _ledger.append(1)

# random() reproducibility under identical seeds
random.seed(123)
a = random.random()
random.seed(123)
b = random.random()
assert a == b; _ledger.append(1)

# uniform() reproducibility under identical seeds
random.seed(42)
u1 = random.uniform(-1, 1)
random.seed(42)
u2 = random.uniform(-1, 1)
assert u1 == u2; _ledger.append(1)

# Consecutive draws within a single seeded stream differ
random.seed(42)
ra1 = random.random()
ra2 = random.random()
assert ra1 != ra2; _ledger.append(1)

# triangular keeps result inside its bounds
random.seed(42)
tr2 = random.triangular(10, 20)
assert 10.0 <= tr2 <= 20.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_random_distribution_extras_ops {sum(_ledger)} asserts")
