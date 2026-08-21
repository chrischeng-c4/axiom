# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "restore_is_callable"
# subject = "difflib.restore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.restore: restore_is_callable (surface)."""
import difflib

assert callable(difflib.restore)
print("restore_is_callable OK")
