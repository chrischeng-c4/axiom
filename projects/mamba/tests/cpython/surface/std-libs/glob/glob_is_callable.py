# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "glob_is_callable"
# subject = "glob.glob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: glob_is_callable (surface)."""
import glob

assert callable(glob.glob)
print("glob_is_callable OK")
