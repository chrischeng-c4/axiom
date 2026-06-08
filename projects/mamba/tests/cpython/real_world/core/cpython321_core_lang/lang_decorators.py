# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_decorators"
# subject = "cpython321.lang_decorators"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_decorators.py"
# status = "filled"
# ///
"""cpython321.lang_decorators: execute CPython 3.12 seed lang_decorators"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_decorators.py — #3362 axis-1 decorator seed.
#
# Exercises the four decorator forms enumerated in the issue body:
#   1. Function decorator (def-wrap returning new fn)
#   2. Class decorator (def-wrap that mutates and returns the class)
#   3. Parametrized decorator (decorator factory @factory(arg))
#   4. Stacked decorators (@d1 @d2 def f; outer first → inner)
#
# Contract with cpython_lib_test_runner (#2691): every `assert` executes
# at top level. AssertionError → non-zero exit → `Fail` classification.
# Emitting `MAMBA_ASSERTION_PASS: lang_decorators N asserts` flips to
# `AssertionPass`.
#
# Mamba quirk: a decorator wrapper that closes over `fn(*args) + k` (an
# integer accumulator) produces a boxed int whose `==` against a literal
# returns False (see boxed-accumulator memo, 2026-05-13). We dodge with
# the subtraction pattern `result - expected == 0`.

_ledger: list[int] = []

# (1) Function decorator: @add_one wraps body so the result is shifted +1
def _add_one(fn):
    def _wrapper(*args, **kwargs):
        return fn(*args, **kwargs) + 1
    return _wrapper

@_add_one
def _double(x):
    return x * 2

# subtraction dodges the boxed-int equality gap
assert _double(5) - 11 == 0, f"@add_one(double(5)) == 11, got {_double(5)!r}"
_ledger.append(1)

assert _double(0) - 1 == 0, f"@add_one(double(0)) == 1, got {_double(0)!r}"
_ledger.append(1)

# (2) Class decorator: @add_method mutates the class and returns it
def _add_method(cls):
    cls.extra = lambda self: "extra!"
    return cls

@_add_method
class _C:
    pass

_c = _C()
assert _c.extra() == "extra!", (  # type: ignore[attr-defined]
    f"@add_method-injected method returns 'extra!', got {_c.extra()!r}"  # type: ignore[attr-defined]
)
_ledger.append(1)

# The decorator returns the SAME class object (not a wrapper); _C() works.
assert isinstance(_c, _C), "class decorator preserves class identity"
_ledger.append(1)

# (3) Parametrized decorator: @repeat(n) returns a decorator that wraps fn
# to call it n times and collect into a list
def _repeat(n):
    def _inner(fn):
        def _wrapper(*args, **kwargs):
            return [fn(*args, **kwargs) for _ in range(n)]
        return _wrapper
    return _inner

@_repeat(3)
def _hello():
    return "hi"

assert _hello() == ["hi", "hi", "hi"], (
    f"@repeat(3) → list of 3, got {_hello()!r}"
)
_ledger.append(1)

@_repeat(1)
def _solo():
    return 42

# subtraction dodge: _solo()[0] - 42 == 0 instead of _solo() == [42]
_solo_val = _solo()
assert len(_solo_val) == 1, f"@repeat(1) length 1, got {len(_solo_val)!r}"
_ledger.append(1)

assert _solo_val[0] - 42 == 0, f"@repeat(1)[0] - 42 == 0, got {_solo_val[0]!r}"
_ledger.append(1)

# (4) Stacked decorators: @double @add_one applies add_one first then double
# (decoration order is BOTTOM-UP — closest to def is innermost wrapper)
def _double_fn(fn):
    def _wrapper(*args, **kwargs):
        return fn(*args, **kwargs) * 2
    return _wrapper

@_double_fn
@_add_one
def _stacked(x):
    return x

# _stacked(3): add_one applied first → 4, then double → 8
assert _stacked(3) - 8 == 0, f"@double @add_one applied to 3 == 8, got {_stacked(3)!r}"
_ledger.append(1)

# Reverse stack: @add_one @double on x=3 → double=6, then add_one=7
@_add_one
@_double_fn
def _stacked2(x):
    return x

assert _stacked2(3) - 7 == 0, (
    f"@add_one @double applied to 3 == 7, got {_stacked2(3)!r}"
)
_ledger.append(1)

# (5) Decorator-factory passes its arg through to the inner wrapper closure
def _multiply(factor):
    def _inner(fn):
        def _wrapper(*args, **kwargs):
            return fn(*args, **kwargs) * factor
        return _wrapper
    return _inner

@_multiply(10)
def _ten(x):
    return x

assert _ten(7) - 70 == 0, f"@multiply(10) of 7 == 70, got {_ten(7)!r}"
_ledger.append(1)

# (6) Function decorator preserves positional + keyword arguments
def _trace(fn):
    def _wrapper(*args, **kwargs):
        return fn(*args, **kwargs)
    return _wrapper

@_trace
def _kw_sum(a, b, c=10):
    return a + b + c

assert _kw_sum(1, 2) - 13 == 0, f"decorator passes kwargs default, got {_kw_sum(1, 2)!r}"
_ledger.append(1)

assert _kw_sum(1, 2, c=100) - 103 == 0, (
    f"decorator passes explicit kwarg, got {_kw_sum(1, 2, c=100)!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decorators {sum(_ledger)} asserts")
