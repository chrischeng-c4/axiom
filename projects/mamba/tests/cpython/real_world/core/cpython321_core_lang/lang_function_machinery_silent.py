# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_function_machinery_silent"
# subject = "cpython321.lang_function_machinery_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_function_machinery_silent.py"
# status = "filled"
# ///
"""cpython321.lang_function_machinery_silent: execute CPython 3.12 seed lang_function_machinery_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of decorator wrap with `*args/**kwargs`
# (the documented "wrap forwards args via fn(*a, **kw) and returns
# the original return value" — mamba forwards arity but the inner
# call returns 0 — the original return-value is dropped), `**dict`
# call-site unpacking (the documented "**{'a':1,'b':2,'c':3} expands
# to a=1,b=2,c=3 keyword args" — mamba passes the dict as the first
# positional arg with b,c=0,0), keyword-only enforcement (the
# documented "def f(a, *, b): f(1, 2) raises TypeError" — mamba
# accepts the positional override), positional-only enforcement (the
# documented "def f(a, b, /): f(a=1, b=2) raises TypeError" — mamba
# accepts the keyword override), function `__defaults__` (the
# documented "callable.__defaults__ exposes the defaults tuple" —
# mamba returns None), `Exception.__mro__` iterability (the
# documented "Exception.__mro__ is an iterable tuple of bases" —
# mamba raises 'object is not iterable'), `*args` parameter type
# (the documented "*args is a tuple" — mamba binds it to a list),
# `callable(1)` (the documented "callable returns False for non-
# callable values" — mamba returns True), `hash([1, 2])` (the
# documented "list is unhashable; hash raises TypeError" — mamba
# silently returns an int hash), and function `__code__` (the
# documented "function.__code__ exposes the bytecode CodeType
# object" — mamba returns None).
# Ten-pack pinned to atomic 324.
#
# Behavioral edges that CONFORM on mamba (closures with nonlocal,
# simple decorators (no *args/**kw in wrap), *args/**kwargs callee-
# side length/index/lookup, *list unpacking call-site, default args
# (positional and keyword), lambdas, late-binding closure capture,
# default-arg early-binding fix, global, function attrs __name__/
# __doc__/__module__/__qualname__, assert-with-message, exception
# chaining __cause__/__context__, issubclass exception hierarchy,
# yield/send coroutine, callable() True cases, hash on hashable
# types, id() identity) are covered in the matching pass fixture
# `test_lang_function_machinery_value_ops`.


_ledger: list[int] = []

# 1) decorator wrap with *args/**kwargs forwards original return value
#    (mamba: returns ("wrapped", 0) instead of ("wrapped", "hi"))
def _dec(fn):
    def _wrap(*a, **kw):
        return ("wrapped", fn(*a, **kw))
    return _wrap

@_dec
def _hello():
    return "hi"
assert _hello() == ("wrapped", "hi"); _ledger.append(1)

# 2) **dict call-site unpacking expands keyword args
#    (mamba: passes the dict as positional, b and c default to 0)
def _f3(a, b, c):
    return (a, b, c)
assert _f3(**{"a": 1, "b": 2, "c": 3}) == (1, 2, 3); _ledger.append(1)

# 3) keyword-only enforcement raises TypeError on positional override
#    (mamba: accepts the call silently)
def _k_only(a, *, b):
    return (a, b)
try:
    _k_only(1, 2)
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 4) positional-only enforcement raises TypeError on keyword override
#    (mamba: accepts the call silently)
def _p_only(a, b, /):
    return (a, b)
try:
    _p_only(a=1, b=2)
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 5) function __defaults__ exposes the defaults tuple
#    (mamba: returns None)
def _with_defaults(x, y=1, z=2):
    return (x, y, z)
assert _with_defaults.__defaults__ == (1, 2); _ledger.append(1)

# 6) Exception.__mro__ is iterable
#    (mamba: raises 'object is not iterable')
_mro_names = [c.__name__ for c in ValueError.__mro__]
assert "Exception" in _mro_names; _ledger.append(1)

# 7) *args parameter type is tuple
#    (mamba: binds *args to list)
def _f_star(*args):
    return type(args).__name__
assert _f_star(1, 2) == "tuple"; _ledger.append(1)

# 8) callable(1) returns False (int is not callable)
#    (mamba: returns True)
assert callable(1) == False; _ledger.append(1)

# 9) hash([1, 2]) raises TypeError (list is unhashable)
#    (mamba: silently returns an int hash)
try:
    hash([1, 2])
    raise AssertionError("expected TypeError")
except TypeError:
    _ledger.append(1)

# 10) function __code__ exposes the bytecode CodeType object
#     (mamba: returns None)
def _fn_with_code():
    pass
assert _fn_with_code.__code__ is not None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_function_machinery_silent {sum(_ledger)} asserts")
