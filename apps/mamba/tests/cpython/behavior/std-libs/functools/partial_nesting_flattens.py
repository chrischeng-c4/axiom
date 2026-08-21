# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_nesting_flattens"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: a partial of a partial flattens: func resolves to the base callable and the merged signature equals a single flat partial"""
import functools


def capture(*args, **kw):
    return (args, kw)


def signature(part):
    return (part.func, part.args, part.keywords)


inner = functools.partial(capture, "asdf")
nested = functools.partial(inner, bar=True)
flat = functools.partial(capture, "asdf", bar=True)

assert nested.func is capture, "nested partial flattened func"
assert signature(nested) == signature(flat), "nested == flat signature"

print("partial_nesting_flattens OK")
