"""Behavior contract for language closures.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: Closure captures variable by reference, not by value
def _make_ref_closure():
    x = 1
    def _get():
        return x
    x = 99
    return _get

assert _make_ref_closure()() == 99, "closure should capture by reference"

# Rule 2: nonlocal allows mutation of enclosing variable
def _make_accumulator(start: int):
    total = start
    def _add(n: int) -> int:
        nonlocal total
        total += n
        return total
    return _add

_acc = _make_accumulator(0)
assert _acc(5) == 5, f"acc(5) = {_acc(5)!r}"
assert _acc(3) == 8, f"acc(3) = {_acc(3)!r}"
assert _acc(2) == 10, f"acc(2) = {_acc(2)!r}"

# Rule 3: Each call to factory creates independent closure
def _make_adder(n: int):
    def _add(x: int) -> int:
        return x + n
    return _add

_add1 = _make_adder(1)
_add2 = _make_adder(2)
assert _add1(10) == 11, f"add1(10) = {_add1(10)!r}"
assert _add2(10) == 12, f"add2(10) = {_add2(10)!r}"
assert _add1.__closure__ is not _add2.__closure__, "closures share __closure__"

# Rule 4: Closure works across multiple levels of nesting
def _outer(a: int):
    def _middle(b: int):
        def _inner(c: int) -> int:
            return a + b + c
        return _inner
    return _middle

assert _outer(1)(2)(3) == 6, f"nested closure = {_outer(1)(2)(3)!r}"

# Rule 5: Closure in class method
class _Greeter:
    def __init__(self, prefix: str):
        self._prefix = prefix

    def make_greet(self):
        prefix = self._prefix
        def _greet(name: str) -> str:
            return f"{prefix}, {name}"
        return _greet

g = _Greeter("Hello")
_greet = g.make_greet()
assert _greet("Alice") == "Hello, Alice", f"greet = {_greet('Alice')!r}"

# Rule 6: functools.wraps preserves __wrapped__ through closure
import functools

def _twice(fn):
    @functools.wraps(fn)
    def _w(*args, **kwargs):
        return fn(*args, **kwargs) * 2
    return _w

@_twice
def _val(n: int) -> int:
    return n

assert _val(5) == 10, f"twice(5) = {_val(5)!r}"
assert _val.__wrapped__ is not None, "__wrapped__ missing"

# Rule 7: Cell contents accessible at runtime
def _make_cell_holder(v: int):
    def _get() -> int:
        return v
    return _get

_f = _make_cell_holder(42)
assert _f.__closure__[0].cell_contents == 42, f"cell = {_f.__closure__[0].cell_contents!r}"

# Rule 8: Lambda can form closures
def _make_multiplier(factor: int):
    return lambda x: x * factor

_triple = _make_multiplier(3)
assert _triple(7) == 21, f"triple(7) = {_triple(7)!r}"
assert _triple.__closure__ is not None, "lambda closure is None"

print("behavior OK")
