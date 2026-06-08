# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "get_overloads_is_callable"
# subject = "typing.get_overloads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_overloads: get_overloads_is_callable (surface)."""
import typing

assert callable(typing.get_overloads)
print("get_overloads_is_callable OK")
