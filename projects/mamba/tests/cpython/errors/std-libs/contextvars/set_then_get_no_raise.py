# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "set_then_get_no_raise"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: set() then get() on a no-default ContextVar returns the set value without raising"""
import contextvars

cv = contextvars.ContextVar("no_default")
cv.set("hello")
# No default, but a value is now set, so get() returns it (no LookupError).
assert cv.get() == "hello", "get() after set() returns the set value"
print("set_then_get_no_raise OK")
