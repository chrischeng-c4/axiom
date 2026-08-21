# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_binds_positional_and_keyword"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial binds leading positional args and default keywords; call-time positionals append and call-time kwargs win on conflict"""
import functools


def capture(*args, **kw):
    return (args, kw)


# Bound positionals lead; call-time positionals append; call-time kwargs win.
p = functools.partial(capture, 1, 2, a=10, b=20)
assert p(3, 4, b=30, c=40) == ((1, 2, 3, 4), {"a": 10, "b": 30, "c": 40}), (
    f"merged call = {p(3, 4, b=30, c=40)!r}"
)


# A bound leading positional, exercised over a 3-arg function.
def _add(a, b, c):
    return a + b + c


assert functools.partial(_add, 5)(1, 2) == 8, "partial(add, 5)(1, 2)"
assert functools.partial(_add, 5, 6)(7) == 18, "partial(add, 5, 6)(7)"


# A bound default keyword.
def _greet(name, greeting="hi"):
    return f"{greeting} {name}"


assert functools.partial(_greet, greeting="hello")("Alice") == "hello Alice", "partial kw"

print("partial_binds_positional_and_keyword OK")
