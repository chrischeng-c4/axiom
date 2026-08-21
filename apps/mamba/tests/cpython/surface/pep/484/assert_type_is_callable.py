# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "assert_type_is_callable"
# subject = "typing.assert_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.assert_type: assert_type_is_callable (surface)."""
import typing

assert callable(typing.assert_type)
print("assert_type_is_callable OK")
