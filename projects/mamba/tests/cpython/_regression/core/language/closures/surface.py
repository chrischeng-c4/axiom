"""Surface contract for language closures.

# type-regime: monomorphic

Probes: closure creation, __closure__ attribute, cell objects,
free variable capture, nonlocal, nested functions.
CPython 3.12 is the oracle.
"""

# Closure captures free variable
def _make_adder(n: int):
    def _add(x: int) -> int:
        return x + n
    return _add

_add5 = _make_adder(5)
assert callable(_add5), "closure not callable"
assert _add5(3) == 8, f"closure result = {_add5(3)!r}"

# __closure__ attribute exists and is a tuple
assert _add5.__closure__ is not None, "__closure__ is None"
assert isinstance(_add5.__closure__, tuple), f"__closure__ type = {type(_add5.__closure__)!r}"
assert len(_add5.__closure__) == 1, f"__closure__ len = {len(_add5.__closure__)!r}"

# Cell objects have cell_contents
_cell = _add5.__closure__[0]
assert hasattr(_cell, "cell_contents"), "cell has no cell_contents"
assert _cell.cell_contents == 5, f"cell_contents = {_cell.cell_contents!r}"

# Non-closure function has __closure__ == None
def _plain() -> int:
    return 0
assert _plain.__closure__ is None, f"non-closure __closure__ = {_plain.__closure__!r}"

# nonlocal rebinding works
def _make_counter():
    _count = 0
    def _inc() -> int:
        nonlocal _count
        _count += 1
        return _count
    return _inc

_c = _make_counter()
assert _c() == 1, f"counter first = {_c()!r}"

# Multiple closures over same variable are independent
_a = _make_adder(10)
_b = _make_adder(20)
assert _a(0) == 10, f"a(0) = {_a(0)!r}"
assert _b(0) == 20, f"b(0) = {_b(0)!r}"

# Closure over multiple free variables
def _make_linear(m: int, b: int):
    def _f(x: int) -> int:
        return m * x + b
    return _f

_line = _make_linear(2, 3)
assert _line(4) == 11, f"linear(4) = {_line(4)!r}"

print("surface OK")
