# Operational AssertionPass seed for `random` continuous distributions
# not covered by `test_random_distribution_ops` /
# `test_random_distribution_extras_ops` (those cover gauss,
# triangular, expovariate). This seed asserts the support-set
# invariants of `paretovariate` (>1 for shape > 0), `weibullvariate`
# (>0 for positive scale/shape), `lognormvariate` (>0 always),
# `gammavariate` (>0 for positive shape/scale), `betavariate`
# (in [0, 1]), `normalvariate` (finite real), and `getrandbits(n)`
# (non-negative int strictly less than 2**n).
import random
_ledger: list[int] = []

# Pareto — support (1, +inf), so always > 1 for positive shape
random.seed(1)
p = random.paretovariate(2.0)
assert isinstance(p, float); _ledger.append(1)
assert p >= 1.0; _ledger.append(1)

# Weibull — support [0, +inf)
random.seed(2)
w = random.weibullvariate(1.0, 1.5)
assert isinstance(w, float); _ledger.append(1)
assert w >= 0.0; _ledger.append(1)

# Lognormal — support (0, +inf)
random.seed(3)
ln = random.lognormvariate(0.0, 1.0)
assert isinstance(ln, float); _ledger.append(1)
assert ln > 0.0; _ledger.append(1)

# Gamma — support (0, +inf) for positive shape/scale
random.seed(4)
g = random.gammavariate(2.0, 1.0)
assert isinstance(g, float); _ledger.append(1)
assert g > 0.0; _ledger.append(1)

# Beta — support [0, 1]
random.seed(5)
b = random.betavariate(2.0, 5.0)
assert isinstance(b, float); _ledger.append(1)
assert 0.0 <= b <= 1.0; _ledger.append(1)

# Normal — finite real (no support restriction)
random.seed(6)
n = random.normalvariate(0.0, 1.0)
assert isinstance(n, float); _ledger.append(1)

# getrandbits — non-negative int strictly less than 2**n
random.seed(7)
g8 = random.getrandbits(8)
assert isinstance(g8, int); _ledger.append(1)
assert 0 <= g8 < 256; _ledger.append(1)

random.seed(8)
g16 = random.getrandbits(16)
assert 0 <= g16 < 65536; _ledger.append(1)

random.seed(9)
g1 = random.getrandbits(1)
assert g1 in (0, 1); _ledger.append(1)

# Seed determinism across distributions
random.seed(42)
a1 = random.paretovariate(2.0)
random.seed(42)
a2 = random.paretovariate(2.0)
assert a1 == a2; _ledger.append(1)

random.seed(42)
b1 = random.gammavariate(2.0, 1.0)
random.seed(42)
b2 = random.gammavariate(2.0, 1.0)
assert b1 == b2; _ledger.append(1)

random.seed(42)
c1 = random.lognormvariate(0.0, 1.0)
random.seed(42)
c2 = random.lognormvariate(0.0, 1.0)
assert c1 == c2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_random_distributions_extras_ops {sum(_ledger)} asserts")
