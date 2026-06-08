# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Function `__doc__` was returning None for top-level defs even when the body
# started with a string literal. CPython exposes the literal as the function's
# docstring; functions without a leading string literal carry __doc__ == None.
# The fix scans each top-level HirFunction body for a leading
# `HirStmt::Expr(HirExpr::StrLit(...))` and primes a FUNC_DOCS thread-local
# registry parallel to FUNC_NAMES, exposed through `mb_getattr` for __doc__.

def f():
    """triple-quoted doc"""
    pass

def g():
    pass

def h():
    "single-line doc"
    return 1

print(f.__doc__)              # triple-quoted doc
print(g.__doc__)              # None
print(h.__doc__)              # single-line doc

# Multiple defs all register independently — no cross-talk.
def a():
    """alpha"""
    pass

def b():
    """beta"""
    pass

print(a.__doc__)              # alpha
print(b.__doc__)              # beta

# __name__ + __doc__ stay consistent on the same function.
print(f.__name__, "->", f.__doc__)   # f -> triple-quoted doc
