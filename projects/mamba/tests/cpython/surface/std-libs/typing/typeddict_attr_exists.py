# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "typeddict_attr_exists"
# subject = "typing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: typeddict_attr_exists (surface)."""
import typing

assert hasattr(typing, "TypedDict")
print("typeddict_attr_exists OK")
