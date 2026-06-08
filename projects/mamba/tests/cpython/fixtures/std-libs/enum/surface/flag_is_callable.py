# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "flag_is_callable"
# subject = "enum.Flag"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.Flag: flag_is_callable (surface)."""
import enum

assert callable(enum.Flag)
print("flag_is_callable OK")
