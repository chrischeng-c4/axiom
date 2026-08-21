# Operational AssertionPass seed for generator semantics.
# Surface: simple yield-from-loop, infinite generator + next(),
# generator expression, dict comprehension, set comprehension,
# yield-from delegation.
# Companion to stub/test_generator.py — vendored unittest seed.
_ledger: list[int] = []

def gen_n(n: int):
    for i in range(n):
        yield i * 10

assert list(gen_n(4)) == [0, 10, 20, 30]; _ledger.append(1)

def naturals():
    i = 1
    while True:
        yield i
        i += 1

g = naturals()
got = [next(g) for _ in range(5)]
assert got == [1, 2, 3, 4, 5]; _ledger.append(1)

ge = (x * x for x in range(5))
assert list(ge) == [0, 1, 4, 9, 16]; _ledger.append(1)

sq = {n: n * n for n in range(4)}
assert sq == {0: 0, 1: 1, 2: 4, 3: 9}; _ledger.append(1)

ss = {x % 3 for x in range(10)}
assert sorted(ss) == [0, 1, 2]; _ledger.append(1)

def src():
    yield 1
    yield 2
    yield 3
def wrap():
    yield from src()
    yield 99

assert list(wrap()) == [1, 2, 3, 99]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_generator_ops {sum(_ledger)} asserts")
