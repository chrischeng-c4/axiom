# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "get_origin_is_callable"
# subject = "typing.get_origin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_origin: get_origin_is_callable (surface)."""
import typing

assert callable(typing.get_origin)
print("get_origin_is_callable OK")
