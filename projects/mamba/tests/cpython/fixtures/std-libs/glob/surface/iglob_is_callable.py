# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "iglob_is_callable"
# subject = "glob.iglob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.iglob: iglob_is_callable (surface)."""
import glob

assert callable(glob.iglob)
print("iglob_is_callable OK")
