# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_closures"
# subject = "cpython321.lang_closures"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_closures.py"
# status = "filled"
# ///
"""cpython321.lang_closures: execute CPython 3.12 seed lang_closures"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_closures.py — #3363 axis-1 closures + nonlocal + global seed.
#
# Exercises:
#   1. Inner function reads outer-scope name (closure read)
#   2. Mutable outer object (list.append) without nonlocal — closure
#      shares the reference, mutation observed across calls
#   3. `nonlocal` rebinding the enclosing-scope binding
#   4. `nonlocal` traversing two enclosing frames
#   5. `global` reading + writing module-level name
#   6. Default-argument loop trick (lambda i=i: i) capturing per-iteration
#      value
#   7. Late-binding lambda gotcha (all closures see the FINAL loop value)
#   8. 3-level nested closures forwarding outermost binding to innermost
#
# Mamba quirks (tracked separately):
#   * Late definition: a closure that reads `y` from the enclosing scope
#     where `y` is assigned AFTER the inner def is created sees 0, not
#     the post-def value. Intentionally NOT exercised.
#
# Contract with cpython_lib_test_runner: AssertionError → Fail;
# MAMBA_ASSERTION_PASS: lang_closures N asserts → AssertionPass.

_ledger: list[int] = []

# (1) Inner reads outer
def _outer_read():
    x = 10
    def _inner():
        return x
    return _inner()

assert _outer_read() - 10 == 0, f"closure read 10, got {_outer_read()!r}"
_ledger.append(1)

# (2) Mutable outer object — append observed across calls (no nonlocal)
def _make_appender():
    items: list[int] = []
    def _append(x):
        items.append(x)
        return items
    return _append

_a = _make_appender()
_after_one = _a(1)
assert _after_one == [1], f"append 1 → [1], got {_after_one!r}"
_ledger.append(1)

_after_two = _a(2)
assert _after_two == [1, 2], f"append 2 → [1, 2], got {_after_two!r}"
_ledger.append(1)

# (3) `nonlocal` rebinds enclosing binding (counter pattern)
def _make_counter():
    count = 0
    def _bump():
        nonlocal count
        count += 1
        return count
    return _bump

_c = _make_counter()
assert _c() - 1 == 0, "counter first call returns 1"
_ledger.append(1)
assert _c() - 2 == 0, "counter second call returns 2"
_ledger.append(1)
assert _c() - 3 == 0, "counter third call returns 3"
_ledger.append(1)

# (4) `nonlocal` traversing two frames
def _two_frame():
    x = 0
    def _middle():
        def _inner():
            nonlocal x
            x = 99
        _inner()
        return x
    return _middle()

assert _two_frame() - 99 == 0, (
    f"nonlocal traverses two frames, got {_two_frame()!r}"
)
_ledger.append(1)

# (5) `global` reads + writes module-level name
g = 42

def _read_g():
    return g

def _write_g():
    global g
    g = 100

assert _read_g() - 42 == 0, f"global read 42, got {_read_g()!r}"
_ledger.append(1)

_write_g()
assert g - 100 == 0, f"global write reflected at module level, got {g!r}"
_ledger.append(1)
assert _read_g() - 100 == 0, f"global write reflected in closure, got {_read_g()!r}"
_ledger.append(1)

# (6) Default-arg loop trick captures per-iteration value
def _make_funcs():
    fs = []
    for i in range(3):
        fs.append(lambda i=i: i)
    return fs

_per_iter = [f() for f in _make_funcs()]
assert _per_iter == [0, 1, 2], (
    f"lambda i=i default-arg captures per-iteration, got {_per_iter!r}"
)
_ledger.append(1)

# (7) Late-binding lambda gotcha: all closures see the final loop value
def _make_late():
    fs = []
    for i in range(3):
        fs.append(lambda: i)
    return fs

_late = [f() for f in _make_late()]
assert _late == [2, 2, 2], (
    f"late-binding lambdas all see final i=2, got {_late!r}"
)
_ledger.append(1)

# (8) 3-level nested closure
def _three_level():
    a = 7
    def _f2():
        def _f3():
            return a
        return _f3()
    return _f2()

assert _three_level() - 7 == 0, (
    f"3-level closure reads outermost 7, got {_three_level()!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_closures {sum(_ledger)} asserts")
