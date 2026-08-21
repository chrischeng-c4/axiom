# Operational AssertionPass seed for random module surfaces beyond
# test_random_seeded_ops (which covers seed determinism for random,
# randint, choice, uniform).
# Surface: sample (without replacement preserves cardinality);
# shuffle is in-place and only permutes; randrange honors step;
# gauss returns a float; randint/choice respect their bounds;
# random returns a value in [0, 1); uniform returns a value in
# [a, b].
import random
_ledger: list[int] = []

random.seed(2026)

# random.random() returns a float in [0, 1)
v = random.random()
assert isinstance(v, float); _ledger.append(1)
assert 0 <= v < 1; _ledger.append(1)

# random.randint(a, b) is inclusive on both ends
for _ in range(20):
    ri = random.randint(1, 10)
    assert 1 <= ri <= 10
_ledger.append(1)

# random.choice picks an element from the sequence
ch = random.choice([100, 200, 300])
assert ch in (100, 200, 300); _ledger.append(1)

# random.uniform(a, b) returns a float in [a, b]
uv = random.uniform(0, 100)
assert isinstance(uv, float); _ledger.append(1)
assert 0 <= uv <= 100; _ledger.append(1)

# random.sample(population, k) draws k unique items
src = [1, 2, 3, 4, 5]
sm = random.sample(src, 3)
assert len(sm) == 3; _ledger.append(1)
# Without replacement — no duplicates in the result
assert len(set(sm)) == 3; _ledger.append(1)
# Every sampled item came from the population
for x in sm:
    assert x in src
_ledger.append(1)

# random.shuffle permutes a list in place — same elements, same length
lst = [1, 2, 3, 4, 5]
original_sorted = sorted(lst)
random.shuffle(lst)
assert len(lst) == 5; _ledger.append(1)
# Set of items is preserved (sort to compare independent of order)
assert sorted(lst) == original_sorted; _ledger.append(1)

# random.randrange(start, stop, step) snaps to the step grid
rg = random.randrange(0, 100, 5)
assert 0 <= rg < 100; _ledger.append(1)
assert rg % 5 == 0; _ledger.append(1)

# random.gauss(mu, sigma) returns a float drawn from a Gaussian
gs = random.gauss(0, 1)
assert isinstance(gs, float); _ledger.append(1)

# Seeded determinism: same seed gives same random() result
random.seed(42)
a = random.random()
random.seed(42)
b = random.random()
assert a == b; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_random_distribution_ops {sum(_ledger)} asserts")
