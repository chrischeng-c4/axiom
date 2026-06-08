# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "surface"
# case = "typealiastype_exists"
# subject = "typing.TypeAliasType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: typealiastype_exists (surface)."""
import typing

assert callable(typing.TypeAliasType)
print("typealiastype_exists OK")
