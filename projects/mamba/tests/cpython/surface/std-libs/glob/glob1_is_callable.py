# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "glob1_is_callable"
# subject = "glob.glob1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob1: glob1_is_callable (surface)."""
import glob

assert callable(glob.glob1)
print("glob1_is_callable OK")
