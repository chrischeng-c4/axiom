# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "displayhook_is_callable"
# subject = "sys.__displayhook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__displayhook__: displayhook_is_callable (surface)."""
import sys

assert callable(sys.__displayhook__)
print("displayhook_is_callable OK")
