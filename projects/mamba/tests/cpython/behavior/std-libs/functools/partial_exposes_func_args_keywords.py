# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_exposes_func_args_keywords"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial.func/.args/.keywords expose the captured callable and bound arguments"""
import functools


def capture(*args, **kw):
    return (args, kw)


p = functools.partial(capture, 1, 2, a=10, b=20)
assert p.func is capture, "partial.func"
assert p.args == (1, 2), f"partial.args = {p.args!r}"
assert p.keywords == {"a": 10, "b": 20}, f"partial.keywords = {p.keywords!r}"

# A keyword-only binding leaves args empty.
q = functools.partial(max, 0)
assert q.func is max, "partial.func builtin"
assert q.args == (0,), f"partial.args = {q.args!r}"

print("partial_exposes_func_args_keywords OK")
