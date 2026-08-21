# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_generator_stateful_yieldfrom_ops"
# subject = "cpython321.test_generator_stateful_yieldfrom_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_generator_stateful_yieldfrom_ops.py"
# status = "filled"
# ///
"""cpython321.test_generator_stateful_yieldfrom_ops: execute CPython 3.12 seed test_generator_stateful_yieldfrom_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for stateful generators and
# `yield from` over heterogeneous source iterables. Surface not
# covered by `test_generator_ops` (yield-loop, generator-expression,
# yield-from over a generator function) or `lang_generator_methods`
# (send / StopIteration.value). This seed asserts: a generator
# preserves locally-bound state across yields (fibonacci-style
# `a, b = b, a + b`); a generator with a default parameter binds the
# default at call-site; `next(g, default)` on an exhausted generator
# returns the default instead of raising; `yield from <list-literal>`
# and `yield from range(N)` both work and can be combined in one
# generator; an early-`return` exit before the first `yield` (or with
# no yields executed) produces an empty iterable; building a list of
# fibonacci values via repeated next() recovers the canonical
# sequence; sum/list/tuple consumers all drain a generator the same
# way.
_ledger: list[int] = []


# 1. Stateful generator — fibonacci preserves (a, b) across yields
def fib():
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b


g = fib()
assert next(g) == 0; _ledger.append(1)
assert next(g) == 1; _ledger.append(1)
assert next(g) == 1; _ledger.append(1)
assert next(g) == 2; _ledger.append(1)
assert next(g) == 3; _ledger.append(1)
assert next(g) == 5; _ledger.append(1)
assert next(g) == 8; _ledger.append(1)
assert next(g) == 13; _ledger.append(1)


# 2. Take-N helper over an infinite generator — list-comprehension-style
def take(gen, n):
    result = []
    for _ in range(n):
        result.append(next(gen))
    return result


fib2 = fib()
assert take(fib2, 10) == [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]; _ledger.append(1)


# 3. Generator with a default parameter — binding captured at call
def counter(start=0):
    n = start
    while True:
        yield n
        n += 1


c0 = counter()
assert next(c0) == 0; _ledger.append(1)
assert next(c0) == 1; _ledger.append(1)
assert next(c0) == 2; _ledger.append(1)

c10 = counter(10)
assert next(c10) == 10; _ledger.append(1)
assert next(c10) == 11; _ledger.append(1)

# Two counter instances are independent
c_a = counter()
c_b = counter(100)
assert next(c_a) == 0; _ledger.append(1)
assert next(c_b) == 100; _ledger.append(1)
assert next(c_a) == 1; _ledger.append(1)
assert next(c_b) == 101; _ledger.append(1)


# 4. next(g, default) on a generator — graceful exhaustion
def two_then_done():
    yield "a"
    yield "b"


td = two_then_done()
assert next(td) == "a"; _ledger.append(1)
assert next(td) == "b"; _ledger.append(1)
# Now exhausted — next(td, default) returns default
assert next(td, "DONE") == "DONE"; _ledger.append(1)
assert next(td, "DONE") == "DONE"; _ledger.append(1)
# A fresh generator with default doesn't return default until exhausted
td2 = two_then_done()
assert next(td2, "DONE") == "a"; _ledger.append(1)
assert next(td2, "DONE") == "b"; _ledger.append(1)
assert next(td2, "DONE") == "DONE"; _ledger.append(1)


# 5. yield from list-literal and yield from range — heterogeneous
def chained_sources():
    yield from [10, 20, 30]
    yield from range(40, 43)
    yield from (50, 51)


assert list(chained_sources()) == [10, 20, 30, 40, 41, 42, 50, 51]; _ledger.append(1)


# 6. yield from over an empty list is a no-op
def with_empty():
    yield from []
    yield 1
    yield from []
    yield 2


assert list(with_empty()) == [1, 2]; _ledger.append(1)


# 7. Generator with early return — produces empty iterable
def early_return():
    if True:
        return
    yield 99  # unreachable


assert list(early_return()) == []; _ledger.append(1)


# 8. Generator that yields then returns — StopIteration after final yield
def yield_then_done():
    yield "x"
    yield "y"
    return


ytd = yield_then_done()
assert next(ytd) == "x"; _ledger.append(1)
assert next(ytd) == "y"; _ledger.append(1)
try:
    next(ytd)
    raise AssertionError("exhausted generator should raise StopIteration")
except StopIteration:
    _ledger.append(1)


# 9. Different consumers drain a generator the same way
def small_gen():
    yield 1
    yield 2
    yield 3


# Each call to small_gen() returns a fresh iterator; consumers don't
# share state
assert list(small_gen()) == [1, 2, 3]; _ledger.append(1)
assert tuple(small_gen()) == (1, 2, 3); _ledger.append(1)
assert sum(small_gen()) == 6; _ledger.append(1)
assert sorted(small_gen()) == [1, 2, 3]; _ledger.append(1)
assert set(small_gen()) == {1, 2, 3}; _ledger.append(1)
assert max(small_gen()) == 3; _ledger.append(1)
assert min(small_gen()) == 1; _ledger.append(1)


# 10. for-loop consumes a generator
def squares(n):
    for i in range(n):
        yield i * i


collected: list[int] = []
for v in squares(5):
    collected.append(v)
assert collected == [0, 1, 4, 9, 16]; _ledger.append(1)


# 11. Generator expression with filter+map combined inline
gen = (x * 2 for x in range(10) if x % 3 == 0)
assert list(gen) == [0, 6, 12, 18]; _ledger.append(1)


# 12. Nested generator — outer yields from inner
def inner(n):
    for i in range(n):
        yield i

def outer():
    yield from inner(3)
    yield from inner(2)


assert list(outer()) == [0, 1, 2, 0, 1]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_generator_stateful_yieldfrom_ops {sum(_ledger)} asserts")
