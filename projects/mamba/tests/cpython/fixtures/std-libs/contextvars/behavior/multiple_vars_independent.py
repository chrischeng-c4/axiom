# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "behavior"
# case = "multiple_vars_independent"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: two distinct ContextVars hold independent values; setting one does not change the other"""
import contextvars

a = contextvars.ContextVar("a", default=1)
b = contextvars.ContextVar("b", default=2)
assert a.get() != b.get(), "distinct defaults"
a.set(10)
assert a.get() == 10, "a updated"
assert b.get() == 2, "b unaffected by a.set"
print("multiple_vars_independent OK")
