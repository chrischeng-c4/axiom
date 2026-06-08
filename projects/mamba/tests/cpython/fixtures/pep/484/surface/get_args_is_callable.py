# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "get_args_is_callable"
# subject = "typing.get_args"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_args: get_args_is_callable (surface)."""
import typing

assert callable(typing.get_args)
print("get_args_is_callable OK")
