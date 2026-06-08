# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "surface"
# case = "typevar_exists"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: typevar_exists (surface)."""
import typing

assert callable(typing.TypeVar)
print("typevar_exists OK")
