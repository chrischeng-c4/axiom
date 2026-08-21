# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "intflag_is_callable"
# subject = "enum.IntFlag"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.IntFlag: intflag_is_callable (surface)."""
import enum

assert callable(enum.IntFlag)
print("intflag_is_callable OK")
