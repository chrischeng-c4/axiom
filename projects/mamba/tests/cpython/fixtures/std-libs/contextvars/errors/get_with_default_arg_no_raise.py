# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "get_with_default_arg_no_raise"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: get(default) on an unset ContextVar returns the supplied default instead of raising LookupError"""
import contextvars

cv = contextvars.ContextVar("no_default")
# Unset and no constructor default, but get(fallback) supplies one at call time.
assert cv.get(99) == 99, "get(default) returns the supplied fallback when unset"
print("get_with_default_arg_no_raise OK")
