# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "auto_is_callable"
# subject = "enum.auto"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.auto: auto_is_callable (surface)."""
import enum

assert callable(enum.auto)
print("auto_is_callable OK")
