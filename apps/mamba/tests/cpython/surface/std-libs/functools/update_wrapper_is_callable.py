# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "update_wrapper_is_callable"
# subject = "functools.update_wrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.update_wrapper: update_wrapper_is_callable (surface)."""
import functools

assert callable(functools.update_wrapper)
print("update_wrapper_is_callable OK")
