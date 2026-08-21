# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "surface"
# case = "generic_exists"
# subject = "typing.Generic"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: generic_exists (surface)."""
import typing

assert hasattr(typing.Generic, "__class_getitem__")
print("generic_exists OK")
