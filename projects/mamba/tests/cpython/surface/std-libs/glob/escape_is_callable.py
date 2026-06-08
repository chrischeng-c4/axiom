# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "escape_is_callable"
# subject = "glob.escape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.escape: escape_is_callable (surface)."""
import glob

assert callable(glob.escape)
print("escape_is_callable OK")
