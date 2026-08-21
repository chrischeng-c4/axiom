"""Surface contract for language functions.

# type-regime: monomorphic

Probes: def, positional args, keyword args, default args, *args, **kwargs,
return value, None return, __name__, callable, closures, lambda.
CPython 3.12 is the oracle.
"""

# Basic function definition and call
def _add(a: int, b: int) -> int:
    return a + b
assert callable(_add), "_add not callable"
assert _add(1, 2) == 3
assert _add.__name__ == "_add", f"__name__ = {_add.__name__!r}"

# Default args
def _greet(name: str, greeting: str = "Hello") -> str:
    return f"{greeting}, {name}!"
assert _greet("Alice") == "Hello, Alice!"
assert _greet("Bob", "Hi") == "Hi, Bob!"

# Keyword args
def _power(base: int, exp: int = 2) -> int:
    return base ** exp
assert _power(3, exp=4) == 81
assert _power(base=2, exp=10) == 1024

# *args
def _sumall(*args: int) -> int:
    return sum(args)
assert _sumall(1, 2, 3) == 6
assert _sumall() == 0

# **kwargs
def _keys(**kwargs: int) -> list:
    return sorted(kwargs.keys())
assert _keys(a=1, b=2, c=3) == ["a", "b", "c"]

# Return None implicitly
def _noop() -> None:
    pass
assert _noop() is None

# Lambda
_sq = lambda x: x * x
assert _sq(5) == 25
assert callable(_sq)

# Closure
def _make_adder(n: int):
    def _adder(x: int) -> int:
        return x + n
    return _adder
add5 = _make_adder(5)
assert add5(3) == 8
assert add5(10) == 15

print("surface OK")
