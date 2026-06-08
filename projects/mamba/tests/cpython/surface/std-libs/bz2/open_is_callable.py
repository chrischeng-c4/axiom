# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "surface"
# case = "open_is_callable"
# subject = "bz2.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bz2.open: open_is_callable (surface)."""
import bz2

assert callable(bz2.open)
print("open_is_callable OK")
