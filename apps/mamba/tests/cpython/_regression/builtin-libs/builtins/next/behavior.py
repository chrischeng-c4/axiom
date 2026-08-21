"""Behavior contract for builtins.next.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: next advances iterator by one
_it = iter([10, 20, 30])
assert next(_it) == 10, f"next() first call = {next(iter([10,20,30]))!r}"

# Rule 2: successive calls advance state
_it = iter([1, 2, 3])
assert next(_it) == 1
assert next(_it) == 2
assert next(_it) == 3

# Rule 3: StopIteration on exhausted iterator
_it = iter([1])
next(_it)
_raised = False
try:
    next(_it)
except StopIteration:
    _raised = True
assert _raised, "next() on exhausted iterator did not raise StopIteration"

# Rule 4: next with default — returns default when exhausted
_it = iter([1])
next(_it)
result = next(_it, "done")
assert result == "done", f"next(exhausted, 'done') = {result!r}"

# Rule 5: next with default — returns next value when not exhausted
_it = iter([42])
result = next(_it, "done")
assert result == 42, f"next(non-exhausted, 'done') = {result!r}"

# Rule 6: next(non-iterator) raises TypeError
_raised = False
try:
    next([1, 2, 3])
except TypeError:
    _raised = True
assert _raised, "next(list) did not raise TypeError"

# Rule 7: next on generator
def _gen():
    yield 1
    yield 2
_g = _gen()
assert next(_g) == 1
assert next(_g) == 2

# Rule 8: StopIteration value is available on exception
def _gen_return():
    yield 1
    return "final"
_g = _gen_return()
next(_g)
_si = None
try:
    next(_g)
except StopIteration as e:
    _si = e
assert _si is not None, "StopIteration not raised"
assert _si.value == "final", \
    f"StopIteration.value = {_si.value!r}, expected 'final'"

# Rule 9: next on range iterator
_it = iter(range(3))
assert next(_it) == 0
assert next(_it) == 1

# Rule 10: default can be None
_it = iter([])
result = next(_it, None)
assert result is None, f"next(empty_iter, None) = {result!r}"

print("behavior OK")
