# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "overload_is_callable"
# subject = "typing.overload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.overload: overload_is_callable (surface)."""
import typing

assert callable(typing.overload)
print("overload_is_callable OK")
