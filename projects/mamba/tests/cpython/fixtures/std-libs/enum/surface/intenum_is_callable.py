# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "intenum_is_callable"
# subject = "enum.IntEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.IntEnum: intenum_is_callable (surface)."""
import enum

assert callable(enum.IntEnum)
print("intenum_is_callable OK")
