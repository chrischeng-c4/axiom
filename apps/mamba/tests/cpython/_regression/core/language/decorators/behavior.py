"""Behavior contract for language decorators.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import functools

# Rule 1: decorator replaces function
def _noop(fn):
    return fn
@_noop
def _fn():
    return 42
assert _fn() == 42, "noop decorator"
assert callable(_fn), "decorated fn callable"

# Rule 2: decorator can wrap behavior
_calls = []
def _trace(fn):
    @functools.wraps(fn)
    def _w(*args, **kwargs):
        _calls.append(fn.__name__)
        return fn(*args, **kwargs)
    return _w

@_trace
def _greet(name: str) -> str:
    return f"hi {name}"

assert _greet("Alice") == "hi Alice"
assert _calls == ["_greet"], f"calls = {_calls!r}"

# Rule 3: functools.wraps preserves metadata
assert _greet.__name__ == "_greet", f"__name__ = {_greet.__name__!r}"
assert _greet.__wrapped__ is not None, "__wrapped__ missing"

# Rule 4: decorator with args via factory
def _clamp(lo: int, hi: int):
    def _d(fn):
        @functools.wraps(fn)
        def _w(*args, **kwargs):
            return max(lo, min(hi, fn(*args, **kwargs)))
        return _w
    return _d

@_clamp(0, 10)
def _identity(x: int) -> int:
    return x

assert _identity(5) == 5, "clamp no-op"
assert _identity(-3) == 0, "clamp lo"
assert _identity(15) == 10, "clamp hi"

# Rule 5: stacked decorators apply bottom-up
_order = []
def _mk_tag(name: str):
    def _d(fn):
        @functools.wraps(fn)
        def _w(*args, **kwargs):
            _order.append(name)
            return fn(*args, **kwargs)
        return _w
    return _d

@_mk_tag("first")
@_mk_tag("second")
def _target() -> int:
    return 0
_target()
assert _order == ["first", "second"], f"order = {_order!r}"  # outer first

# Rule 6: class decorator
def _add_method(cls):
    def _hello(self) -> str:
        return f"hello from {type(self).__name__}"
    cls.hello = _hello
    return cls

@_add_method
class _Widget:
    pass

w = _Widget()
assert w.hello() == "hello from _Widget", f"class decorator = {w.hello()!r}"

# Rule 7: property with getter, setter, deleter
class _Box:
    def __init__(self, side: float):
        self._side = side

    @property
    def area(self) -> float:
        return self._side ** 2

    @area.setter
    def area(self, value: float) -> None:
        import math
        self._side = math.sqrt(value)

b = _Box(3.0)
assert b.area == 9.0, f"area = {b.area!r}"
b.area = 16.0
assert abs(b._side - 4.0) < 1e-10, f"_side after set = {b._side!r}"

# Rule 8: functools.lru_cache as decorator
@functools.lru_cache(maxsize=128)
def _fib(n: int) -> int:
    if n <= 1:
        return n
    return _fib(n - 1) + _fib(n - 2)

assert _fib(10) == 55
assert _fib.cache_info().hits > 0, "lru_cache not caching"

print("behavior OK")
