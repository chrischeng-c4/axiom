# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "rmtree_is_callable"
# subject = "shutil.rmtree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.rmtree: rmtree_is_callable (surface)."""
import shutil

assert callable(shutil.rmtree)
print("rmtree_is_callable OK")
