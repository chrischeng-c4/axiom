# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "unique_is_callable"
# subject = "enum.unique"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""enum.unique: unique_is_callable (surface)."""
import enum

assert callable(enum.unique)
print("unique_is_callable OK")
