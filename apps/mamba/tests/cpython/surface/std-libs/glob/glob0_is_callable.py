# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "glob0_is_callable"
# subject = "glob.glob0"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob0: glob0_is_callable (surface)."""
import glob

assert callable(glob.glob0)
print("glob0_is_callable OK")
