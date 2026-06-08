# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "total_ordering_is_callable"
# subject = "functools.total_ordering"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.total_ordering: total_ordering_is_callable (surface)."""
import functools

assert callable(functools.total_ordering)
print("total_ordering_is_callable OK")
