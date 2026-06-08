"""Behavior contract for language generators.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: yield produces values lazily
def _count(n: int):
    for i in range(n):
        yield i
assert list(_count(5)) == [0, 1, 2, 3, 4]

# Rule 2: generator is suspended between yields
def _trace():
    yield 1
    yield 2
g = _trace()
assert next(g) == 1   # paused at yield 2
assert next(g) == 2   # paused at StopIteration

# Rule 3: StopIteration carries return value
def _gen_return():
    yield 1
    return "done"
g2 = _gen_return()
next(g2)
_si = None
try:
    next(g2)
except StopIteration as e:
    _si = e
assert _si is not None
assert _si.value == "done", f"StopIteration.value = {_si.value!r}"

# Rule 4: generator expression
gen_exp = (x * x for x in range(5))
assert list(gen_exp) == [0, 1, 4, 9, 16]

# Rule 5: send() sends value into generator
def _accumulator():
    total = 0
    while True:
        v = yield total
        if v is None:
            break
        total += v
g3 = _accumulator()
next(g3)  # prime
assert g3.send(10) == 10
assert g3.send(20) == 30
assert g3.send(5) == 35

# Rule 6: throw() injects exception into generator
def _catch_gen():
    try:
        yield 1
        yield 2
    except ValueError as e:
        yield f"caught: {e}"
g4 = _catch_gen()
assert next(g4) == 1
result = g4.throw(ValueError("oops"))
assert result == "caught: oops", f"throw result = {result!r}"

# Rule 7: close() causes GeneratorExit inside generator
_closed = False
def _close_gen():
    global _closed
    try:
        yield 1
    except GeneratorExit:
        _closed = True
        raise
g5 = _close_gen()
next(g5)
g5.close()
assert _closed, "GeneratorExit not raised in generator"

# Rule 8: yield from delegates to sub-iterator
def _inner():
    yield 10
    yield 20
def _outer():
    yield 1
    yield from _inner()
    yield 2
assert list(_outer()) == [1, 10, 20, 2]

# Rule 9: infinite generator with islice
def _naturals():
    n = 0
    while True:
        yield n
        n += 1
from itertools import islice
assert list(islice(_naturals(), 5)) == [0, 1, 2, 3, 4]

# Rule 10: generator from class with __iter__ and __next__
class _Counter:
    def __init__(self, stop: int):
        self._n = 0
        self._stop = stop
    def __iter__(self):
        return self
    def __next__(self) -> int:
        if self._n >= self._stop:
            raise StopIteration
        v = self._n
        self._n += 1
        return v
assert list(_Counter(4)) == [0, 1, 2, 3]

print("behavior OK")
