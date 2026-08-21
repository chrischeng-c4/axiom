# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "excepthook_is_callable"
# subject = "sys.__excepthook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__excepthook__: excepthook_is_callable (surface)."""
import sys

assert callable(sys.__excepthook__)
print("excepthook_is_callable OK")
