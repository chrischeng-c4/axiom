# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "newtype_is_callable"
# subject = "typing.NewType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NewType: newtype_is_callable (surface)."""
import typing

assert callable(typing.NewType)
print("newtype_is_callable OK")
