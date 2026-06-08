# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "copy_is_callable"
# subject = "copy.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""copy.copy: copy_is_callable (surface)."""
import copy

assert callable(copy.copy)
print("copy_is_callable OK")
