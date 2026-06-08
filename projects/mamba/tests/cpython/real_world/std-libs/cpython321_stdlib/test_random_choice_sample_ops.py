# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_random_choice_sample_ops"
# subject = "cpython321.test_random_choice_sample_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_random_choice_sample_ops.py"
# status = "filled"
# ///
"""cpython321.test_random_choice_sample_ops: execute CPython 3.12 seed test_random_choice_sample_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the random.choice/sample/shuffle
# surface. Surface: random.seed() makes the generator deterministic;
# random.choice() picks a member of the source sequence; random.sample()
# returns N distinct members of the source; random.shuffle() permutes
# a list in place (length and multiset preserved); random.random()
# returns a value in [0, 1); random.randint(a, b) returns a value in
# the inclusive interval [a, b]; random.randrange(a, b) returns a value
# in [a, b); random.uniform(a, b) returns a value >= a; same seed
# replays identical sequences (the determinism contract). Companion to
# test_random_seeded_ops and test_random_distribution_ops.
import random
_ledger: list[int] = []

# choice — member of source sequence
random.seed(42)
assert random.choice([1, 2, 3, 4, 5]) in [1, 2, 3, 4, 5]; _ledger.append(1)
random.seed(42)
assert random.choice(["a", "b", "c"]) in ["a", "b", "c"]; _ledger.append(1)

# sample — N distinct members of source
random.seed(0)
s = random.sample([1, 2, 3, 4, 5], 3)
assert len(s) == 3; _ledger.append(1)
assert all(x in [1, 2, 3, 4, 5] for x in s); _ledger.append(1)
random.seed(0)
assert len(set(random.sample([1, 2, 3, 4, 5], 3))) == 3; _ledger.append(1)
random.seed(8)
samp = random.sample(range(20), 5)
assert len(samp) == 5; _ledger.append(1)
assert len(set(samp)) == 5; _ledger.append(1)
assert all(0 <= x < 20 for x in samp); _ledger.append(1)

# shuffle — in place, length + multiset preserved
random.seed(1)
lst = [1, 2, 3, 4, 5]
random.shuffle(lst)
assert len(lst) == 5; _ledger.append(1)
assert sorted(lst) == [1, 2, 3, 4, 5]; _ledger.append(1)
random.seed(9)
lst2 = [10, 20, 30, 40]
random.shuffle(lst2)
assert len(lst2) == 4; _ledger.append(1)
assert sorted(lst2) == [10, 20, 30, 40]; _ledger.append(1)

# random() — [0, 1)
random.seed(2)
v = random.random()
assert v >= 0.0; _ledger.append(1)
assert v < 1.0; _ledger.append(1)

# randint(a, b) — inclusive interval
random.seed(3)
r = random.randint(1, 10)
assert r >= 1; _ledger.append(1)
assert r <= 10; _ledger.append(1)
random.seed(33)
r2 = random.randint(0, 0)
assert r2 == 0; _ledger.append(1)

# randrange(a, b) — half-open [a, b)
random.seed(4)
rr = random.randrange(5, 10)
assert rr >= 5; _ledger.append(1)
assert rr < 10; _ledger.append(1)

# uniform — float in [a, b]
random.seed(5)
u = random.uniform(0.0, 1.0)
assert u >= 0.0; _ledger.append(1)

# Determinism — same seed replays the identical first value
random.seed(99)
a = random.random()
random.seed(99)
b = random.random()
assert a == b; _ledger.append(1)
random.seed(99)
c = random.randint(1, 100)
random.seed(99)
d = random.randint(1, 100)
assert c == d; _ledger.append(1)

# Distinct seeds yield (with overwhelming probability) distinct first
# values from the same draw — sanity that the seed actually steers
random.seed(1)
e1 = random.random()
random.seed(2)
e2 = random.random()
assert e1 != e2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_random_choice_sample_ops {sum(_ledger)} asserts")
