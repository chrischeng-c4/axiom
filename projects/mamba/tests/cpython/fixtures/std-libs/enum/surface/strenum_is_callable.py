# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "strenum_is_callable"
# subject = "enum.StrEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.StrEnum: strenum_is_callable (surface)."""
import enum

assert callable(enum.StrEnum)
print("strenum_is_callable OK")
