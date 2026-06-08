# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_copy_shares_members"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: copy.copy(partial) preserves func and shares the same args/keywords objects"""
import copy
import functools


def capture(*args, **kw):
    return (args, kw)


c = functools.partial(capture, ["asdf"], bar=[True])
c_copy = copy.copy(c)
assert c_copy.func is c.func, "copy shares func"
assert c_copy.args is c.args, "copy shares args"
assert c_copy.keywords is c.keywords, "copy shares keywords"

print("partial_copy_shares_members OK")
