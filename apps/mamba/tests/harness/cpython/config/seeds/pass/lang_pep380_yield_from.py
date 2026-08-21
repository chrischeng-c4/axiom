# Operational AssertionPass seed for PEP 380 — `yield from` delegation.
# Surface: a generator can delegate to a sub-generator via
# `yield from <iterable>`, flattening the inner sequence into the
# outer one. Equivalent to `for v in inner(): yield v` but with the
# explicit delegation token.
def inner():
    yield 1
    yield 2
    yield 3

def outer():
    yield 0
    yield from inner()
    yield 4

# yield from an iterable (list) — not just another generator
def from_list():
    yield from [10, 20, 30]

# Nested delegation — yield from a generator that itself delegates
def deepest():
    yield "a"
    yield "b"

def middle():
    yield from deepest()
    yield "c"

def top():
    yield from middle()
    yield "d"

_ledger: list[int] = []
assert list(outer()) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(from_list()) == [10, 20, 30]; _ledger.append(1)
assert list(top()) == ["a", "b", "c", "d"]; _ledger.append(1)
# Length invariants survive delegation
assert len(list(outer())) == 5; _ledger.append(1)
assert len(list(top())) == 4; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep380_yield_from {sum(_ledger)} asserts")
