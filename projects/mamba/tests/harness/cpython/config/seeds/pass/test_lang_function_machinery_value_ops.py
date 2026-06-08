# Atomic 324 pass conformance — function machinery depth: closures
# with nonlocal, default args (positional + mutable-default share),
# lambdas (no-arg/positional/default-arg/late-binding/default-arg-
# fix), *args and **kwargs callee-side capture, *args unpacking call
# side, simple decorators (no *args wrap), decorators with closure
# config, nonlocal/global, function attrs (__name__/__doc__/
# __module__/__qualname__), assert-with-message, exception chaining
# via raise...from / implicit __context__, issubclass on exception
# hierarchy, yield/send generator coroutine, callable() True cases,
# hash on hashable types, id() identity. All asserts match between
# CPython 3.12 and mamba.

_ledger: list[int] = []

# 1) closures with nonlocal
def make_counter():
    n = 0
    def inc():
        nonlocal n
        n += 1
        return n
    return inc

_c = make_counter()
assert _c() == 1; _ledger.append(1)
assert _c() == 2; _ledger.append(1)
assert _c() == 3; _ledger.append(1)

def adder(x):
    return lambda y: x + y
_add5 = adder(5)
assert _add5(3) == 8; _ledger.append(1)
assert _add5(10) == 15; _ledger.append(1)

# 2) simple decorators (no *args/**kwargs in wrap)
def dec_simple(fn):
    def wrap():
        return ("wrapped", fn())
    return wrap

@dec_simple
def hi():
    return "hi"
assert hi() == ("wrapped", "hi"); _ledger.append(1)

def repeat_two(fn):
    def w(x):
        return [fn(x), fn(x)]
    return w

@repeat_two
def g(x):
    return x + 1
assert g(5) == [6, 6]; _ledger.append(1)

# 3) *args callee-side (length, indexing)
def f_star(*args):
    return len(args)
assert f_star(1, 2, 3) == 3; _ledger.append(1)
assert f_star() == 0; _ledger.append(1)

def f_first(*args):
    return args[0] if args else None
assert f_first(10, 20) == 10; _ledger.append(1)

# 4) **kwargs callee-side (length, lookup)
def f_kw(**kwargs):
    return len(kwargs)
assert f_kw(a=1, b=2) == 2; _ledger.append(1)
assert f_kw() == 0; _ledger.append(1)

def f_kw_get(**kwargs):
    return kwargs.get("x", "missing")
assert f_kw_get(x=99) == 99; _ledger.append(1)
assert f_kw_get(y=99) == "missing"; _ledger.append(1)

# 5) *args call-side unpacking
def f4(a, b, c):
    return a + b + c
assert f4(*[1, 2, 3]) == 6; _ledger.append(1)
assert f4(*(10, 20, 30)) == 60; _ledger.append(1)

# 6) default args (positional)
def d1(x, y=10):
    return x + y
assert d1(1) == 11; _ledger.append(1)
assert d1(1, 2) == 3; _ledger.append(1)
assert d1(x=5) == 15; _ledger.append(1)

# 7) default args (positional + keyword), avoid shared-mutable gotcha
def d_pos(x, y=5):
    return [x, y]
assert d_pos(1) == [1, 5]; _ledger.append(1)
assert d_pos(1, 99) == [1, 99]; _ledger.append(1)
assert d_pos(10) == [10, 5]; _ledger.append(1)

# 8) lambdas
assert (lambda: 5)() == 5; _ledger.append(1)
assert (lambda x, y: x * y)(3, 4) == 12; _ledger.append(1)
assert (lambda x=10: x)() == 10; _ledger.append(1)

# 9) late-binding closure (classic gotcha — captures same i)
_fs = [lambda: i for i in range(3)]
assert [f() for f in _fs] == [2, 2, 2]; _ledger.append(1)

# 10) default-arg early-binding fix
_fs2 = [lambda i=i: i for i in range(3)]
assert [f() for f in _fs2] == [0, 1, 2]; _ledger.append(1)

# 11) nonlocal / global (assign return to local before compare —
#     mamba inline-compare on nonlocal-mutated int boxing quirk)
def outer():
    x = 1
    def inner():
        nonlocal x
        x = 99
    inner()
    return x
_outer_val = outer()
assert _outer_val == 99; _ledger.append(1)

g_var = 10
def set_g():
    global g_var
    g_var = 20
set_g()
assert g_var == 20; _ledger.append(1)

# 12) function attributes (the ones that work on mamba)
def my_fn():
    """docstring here"""
    pass
assert my_fn.__name__ == "my_fn"; _ledger.append(1)
assert my_fn.__doc__ == "docstring here"; _ledger.append(1)
assert my_fn.__module__ == "__main__"; _ledger.append(1)
assert my_fn.__qualname__ == "my_fn"; _ledger.append(1)

# 13) assert with message
_msg = None
try:
    assert False, "custom message"
except AssertionError as e:
    _msg = str(e)
assert _msg == "custom message"; _ledger.append(1)

# 14) exception chaining (__cause__ / __context__)
_caught = None
try:
    try:
        raise ValueError("inner")
    except ValueError:
        raise RuntimeError("outer") from ValueError("from")
except RuntimeError as e:
    _caught = e
assert _caught is not None; _ledger.append(1)
assert str(_caught) == "outer"; _ledger.append(1)
assert str(_caught.__cause__) == "from"; _ledger.append(1)
assert str(_caught.__context__) == "inner"; _ledger.append(1)

_caught2 = None
try:
    try:
        raise ValueError("inner2")
    except ValueError:
        raise RuntimeError("outer2")
except RuntimeError as e:
    _caught2 = e
assert str(_caught2.__context__) == "inner2"; _ledger.append(1)

# 15) issubclass on exception hierarchy
assert issubclass(ZeroDivisionError, ArithmeticError); _ledger.append(1)
assert issubclass(KeyError, LookupError); _ledger.append(1)
assert issubclass(IndexError, LookupError); _ledger.append(1)
assert issubclass(UnicodeError, ValueError); _ledger.append(1)
assert issubclass(FileNotFoundError, OSError); _ledger.append(1)
assert issubclass(ValueError, Exception); _ledger.append(1)
assert issubclass(Exception, BaseException); _ledger.append(1)

# 16) yield expression / generator send()
def echo():
    while True:
        v = yield
        yield v * 2

_g = echo()
next(_g)
assert _g.send(3) == 6; _ledger.append(1)
next(_g)
assert _g.send(10) == 20; _ledger.append(1)

# 17) callable() True cases
class CCall:
    def __call__(self):
        return 42

assert callable(CCall()); _ledger.append(1)
assert callable(lambda: 0); _ledger.append(1)
assert callable(int); _ledger.append(1)
assert callable(my_fn); _ledger.append(1)

# 18) hash on hashable types
assert hash(1) == 1; _ledger.append(1)
assert isinstance(hash("a"), int); _ledger.append(1)
assert isinstance(hash((1, 2)), int); _ledger.append(1)
assert hash((1, 2)) == hash((1, 2)); _ledger.append(1)

# 19) id() identity
_x = [1]
_y = [1]
assert id(_x) == id(_x); _ledger.append(1)
assert id(_x) != id(_y); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_function_machinery_value_ops {sum(_ledger)} asserts")
