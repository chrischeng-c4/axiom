# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "get_type_hints_is_callable"
# subject = "typing.get_type_hints"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints_is_callable (surface)."""
import typing

assert callable(typing.get_type_hints)
print("get_type_hints_is_callable OK")
