# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_iter_next_sentinel_ops"
# subject = "cpython321.test_iter_next_sentinel_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_iter_next_sentinel_ops.py"
# status = "filled"
# ///
"""cpython321.test_iter_next_sentinel_ops: execute CPython 3.12 seed test_iter_next_sentinel_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `iter()` / `next()` builtins
# at the entry-point layer — distinct from `lang_iterator_protocol.py`
# (which tests user `__iter__`/`__next__` definitions) and from
# `test_iter_builtins_ops.py` / `test_builtin_iter_helpers_ops.py`
# (which test enumerate/zip/map/filter/reversed but never `iter()` or
# `next()` directly).
#
# Surface: `iter(seq)` returns an iterator that yields items in order;
# `next(it)` advances and returns the next; once the iterator is
# exhausted, `next(it)` raises `StopIteration`; `next(it, default)`
# returns the default instead of raising; the sentinel form
# `iter(callable, sentinel)` calls the callable repeatedly and stops
# when the return value equals sentinel; two iterators built from the
# same iterable advance independently; `iter(dict)` iterates keys;
# `iter(str)` iterates characters.
_ledger: list[int] = []

# iter(list) + next() — basic forward iteration
xs = [10, 20, 30]
it1 = iter(xs)
assert next(it1) == 10; _ledger.append(1)
assert next(it1) == 20; _ledger.append(1)
assert next(it1) == 30; _ledger.append(1)

# Once exhausted, next() raises StopIteration
try:
    next(it1)
    raise AssertionError("next on exhausted iterator must raise StopIteration")
except StopIteration:
    _ledger.append(1)

# next(it, default) returns default on exhaustion instead of raising
assert next(iter([]), -1) == -1; _ledger.append(1)
assert next(iter([42]), -1) == 42; _ledger.append(1)

# After exhaustion, next(it, default) still returns the default
it2 = iter([1])
assert next(it2) == 1; _ledger.append(1)
assert next(it2, "done") == "done"; _ledger.append(1)
# Subsequent calls keep returning the default — exhausted iterators
# stay exhausted
assert next(it2, "done") == "done"; _ledger.append(1)

# Sentinel form: iter(callable, sentinel) calls callable until it
# returns sentinel, then stops
_state = [0]
def _counter():
    _state[0] += 1
    return _state[0]


sit = iter(_counter, 4)  # stops when _counter() returns 4
collected = list(sit)
assert collected == [1, 2, 3]; _ledger.append(1)
# The callable was called exactly four times — three values plus the
# sentinel-hitting fourth
assert _state[0] == 4; _ledger.append(1)

# Sentinel form with immediate sentinel — empty iterator
_state2 = [0]
def _zero():
    _state2[0] += 1
    return 0


sit2 = iter(_zero, 0)
assert list(sit2) == []; _ledger.append(1)
# Callable was called once and matched immediately
assert _state2[0] == 1; _ledger.append(1)

# Two iterators from the same iterable are independent (each carries
# its own cursor)
xs2 = [1, 2, 3]
a = iter(xs2)
b = iter(xs2)
assert next(a) == 1; _ledger.append(1)
assert next(b) == 1; _ledger.append(1)
assert next(a) == 2; _ledger.append(1)
assert next(b) == 2; _ledger.append(1)
assert next(a) == 3; _ledger.append(1)
assert next(b) == 3; _ledger.append(1)

# iter(dict) iterates over keys
d = {"a": 1, "b": 2, "c": 3}
keys = []
itd = iter(d)
keys.append(next(itd))
keys.append(next(itd))
keys.append(next(itd))
assert set(keys) == {"a", "b", "c"}; _ledger.append(1)
# Iterator exhausted after all keys consumed
try:
    next(itd)
    raise AssertionError("dict iter must exhaust")
except StopIteration:
    _ledger.append(1)

# iter(str) iterates over characters
its = iter("abc")
assert next(its) == "a"; _ledger.append(1)
assert next(its) == "b"; _ledger.append(1)
assert next(its) == "c"; _ledger.append(1)
try:
    next(its)
    raise AssertionError("str iter must exhaust")
except StopIteration:
    _ledger.append(1)

# iter(tuple) iterates over elements
itt = iter((10, 20))
assert next(itt) == 10; _ledger.append(1)
assert next(itt) == 20; _ledger.append(1)

# iter(range) iterates over range values
itr = iter(range(3))
assert next(itr) == 0; _ledger.append(1)
assert next(itr) == 1; _ledger.append(1)
assert next(itr) == 2; _ledger.append(1)

# Round-trip — list(iter(x)) recovers the sequence
assert list(iter([5, 6, 7])) == [5, 6, 7]; _ledger.append(1)
assert list(iter("xyz")) == ["x", "y", "z"]; _ledger.append(1)

# Empty-iter forms
assert list(iter([])) == []; _ledger.append(1)
assert list(iter("")) == []; _ledger.append(1)
try:
    next(iter([]))
    raise AssertionError("next on empty iter must raise")
except StopIteration:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_iter_next_sentinel_ops {sum(_ledger)} asserts")
