# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "move_is_callable"
# subject = "shutil.move"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.move: move_is_callable (surface)."""
import shutil

assert callable(shutil.move)
print("move_is_callable OK")
