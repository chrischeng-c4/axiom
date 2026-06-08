# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "copytree_is_callable"
# subject = "shutil.copytree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.copytree: copytree_is_callable (surface)."""
import shutil

assert callable(shutil.copytree)
print("copytree_is_callable OK")
