# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "surface"
# case = "typevartuple_exists"
# subject = "typing.TypeVarTuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVarTuple: typevartuple_exists (surface)."""
import typing

assert callable(typing.TypeVarTuple)
print("typevartuple_exists OK")
