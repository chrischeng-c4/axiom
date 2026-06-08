# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "typevar_is_callable"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: typevar_is_callable (surface)."""
import typing

assert callable(typing.TypeVar)
print("typevar_is_callable OK")
