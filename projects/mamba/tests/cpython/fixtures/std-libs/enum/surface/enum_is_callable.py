# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "enum_is_callable"
# subject = "enum.Enum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.Enum: enum_is_callable (surface)."""
import enum

assert callable(enum.Enum)
print("enum_is_callable OK")
