# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_functools_partial_ops"
# subject = "cpython321.test_functools_partial_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_functools_partial_ops.py"
# status = "filled"
# ///
"""cpython321.test_functools_partial_ops: execute CPython 3.12 seed test_functools_partial_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `functools.partial`.
# Surface: partial binds positional args left-to-right; later calls
# supply the remaining positional args; keyword binding works for
# arbitrary callables.
import functools
_ledger: list[int] = []

def add3(a, b, c):
    return a + b + c

# One pre-bound arg, two supplied at call site
p1 = functools.partial(add3, 1)
r1 = p1(2, 3)
assert r1 == 6; _ledger.append(1)

# Two pre-bound args, one supplied at call site
p2 = functools.partial(add3, 1, 2)
r2 = p2(3)
assert r2 == 6; _ledger.append(1)

# Pre-bound positional arg on a string-returning callable
def greet(greeting, name):
    return f"{greeting}, {name}!"

gp = functools.partial(greet, "Hello")
assert gp("World") == "Hello, World!"; _ledger.append(1)
assert gp("Alice") == "Hello, Alice!"; _ledger.append(1)

# Same partial reused with a different argument
hp = functools.partial(greet, "Hi")
assert hp("Bob") == "Hi, Bob!"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_functools_partial_ops {sum(_ledger)} asserts")
