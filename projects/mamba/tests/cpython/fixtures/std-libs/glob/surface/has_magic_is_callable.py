# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "has_magic_is_callable"
# subject = "glob.has_magic"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.has_magic: has_magic_is_callable (surface)."""
import glob

assert callable(glob.has_magic)
print("has_magic_is_callable OK")
