# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "cast_is_callable"
# subject = "typing.cast"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.cast: cast_is_callable (surface)."""
import typing

assert callable(typing.cast)
print("cast_is_callable OK")
