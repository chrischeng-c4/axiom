"""Surface contract for language decorators.

# type-regime: monomorphic

Probes: function decorator, class decorator, decorator with args,
stacked decorators, functools.wraps, property decorator.
CPython 3.12 is the oracle.
"""

import functools

# Simple function decorator
def _double_result(fn):
    @functools.wraps(fn)
    def _wrapper(*args, **kwargs):
        return fn(*args, **kwargs) * 2
    return _wrapper

@_double_result
def _add(a: int, b: int) -> int:
    return a + b

assert _add(3, 4) == 14, f"decorator result = {_add(3,4)!r}"
assert _add.__name__ == "_add", f"__name__ = {_add.__name__!r}"  # wraps preserves name

# Decorator with arguments
def _repeat(n: int):
    def _decorator(fn):
        @functools.wraps(fn)
        def _wrapper(*args, **kwargs):
            result = None
            for _ in range(n):
                result = fn(*args, **kwargs)
            return result
        return _wrapper
    return _decorator

@_repeat(3)
def _inc(x: int) -> int:
    return x + 1

assert _inc(0) == 1, f"repeat result = {_inc(0)!r}"

# Stacked decorators — applied bottom-up
def _tag(label: str):
    def _d(fn):
        @functools.wraps(fn)
        def _w(*a, **kw):
            return f"[{label}]{fn(*a, **kw)}"
        return _w
    return _d

@_tag("outer")
@_tag("inner")
def _hello(name: str) -> str:
    return f"hello {name}"

result = _hello("world")
assert result == "[outer][inner]hello world", f"stacked = {result!r}"

# property decorator
class _Temp:
    def __init__(self, celsius: float):
        self._c = celsius
    @property
    def fahrenheit(self) -> float:
        return self._c * 9/5 + 32

t = _Temp(100)
assert t.fahrenheit == 212.0, f"property = {t.fahrenheit!r}"

print("surface OK")
