# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "is_typeddict_is_callable"
# subject = "typing.is_typeddict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.is_typeddict: is_typeddict_is_callable (surface)."""
import typing

assert callable(typing.is_typeddict)
print("is_typeddict_is_callable OK")
