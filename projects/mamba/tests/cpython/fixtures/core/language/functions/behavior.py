"""Behavior contract for language functions.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: positional args
def _mul(a: int, b: int) -> int:
    return a * b
assert _mul(3, 4) == 12
assert _mul(0, 100) == 0

# Rule 2: default args are evaluated once at definition time
def _append_to(val: int, lst: list = []) -> list:  # noqa: B006
    lst.append(val)
    return lst
r1 = _append_to(1)
r2 = _append_to(2)
assert r1 is r2, "default list is shared across calls"
assert r1 == [1, 2], f"shared default = {r1!r}"

# Rule 3: keyword args can be in any order
def _sub(a: int, b: int) -> int:
    return a - b
assert _sub(b=3, a=10) == 7
assert _sub(a=5, b=2) == 3

# Rule 4: *args captures extra positionals as tuple
def _variadic(*args: int) -> tuple:
    return args
result = _variadic(1, 2, 3)
assert result == (1, 2, 3), f"*args = {result!r}"
assert type(result) is tuple, f"type(*args) = {type(result).__name__!r}"

# Rule 5: **kwargs captures extra keywords as dict
def _kw(**kwargs: int) -> dict:
    return kwargs
result2 = _kw(x=1, y=2)
assert result2 == {"x": 1, "y": 2}, f"**kwargs = {result2!r}"
assert type(result2) is dict, f"type(**kwargs) = {type(result2).__name__!r}"

# Rule 6: mixed args ordering
def _mixed(a: int, b: int = 0, *args: int, **kwargs: int) -> tuple:
    return (a, b, args, kwargs)
r = _mixed(1, 2, 3, 4, x=10)
assert r == (1, 2, (3, 4), {"x": 10}), f"mixed = {r!r}"

# Rule 7: return exits immediately
def _early_return(n: int) -> int:
    for i in range(n):
        if i > 2:
            return i
    return -1
assert _early_return(10) == 3
assert _early_return(2) == -1

# Rule 8: recursive function
def _fib(n: int) -> int:
    if n <= 1:
        return n
    return _fib(n - 1) + _fib(n - 2)
assert _fib(0) == 0
assert _fib(1) == 1
assert _fib(10) == 55

# Rule 9: closure captures variable by reference
def _counter() -> tuple:
    count = 0
    def _inc() -> int:
        nonlocal count
        count += 1
        return count
    def _get() -> int:
        return count
    return _inc, _get
inc, get = _counter()
assert get() == 0
inc(); inc(); inc()
assert get() == 3

# Rule 10: first-class functions — pass as arg
def _apply(f, x: int) -> int:
    return f(x)
assert _apply(lambda x: x * 2, 5) == 10

# Rule 11: function stored in list
ops = [lambda x: x + 1, lambda x: x * 2, lambda x: x ** 2]
assert [f(3) for f in ops] == [4, 6, 9], "functions in list"

print("behavior OK")
