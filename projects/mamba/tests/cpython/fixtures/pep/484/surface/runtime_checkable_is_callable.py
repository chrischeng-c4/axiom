# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "runtime_checkable_is_callable"
# subject = "typing.runtime_checkable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: runtime_checkable_is_callable (surface)."""
import typing

assert callable(typing.runtime_checkable)
print("runtime_checkable_is_callable OK")
