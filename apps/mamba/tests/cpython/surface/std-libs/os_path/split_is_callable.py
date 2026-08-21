# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "split_is_callable"
# subject = "os.path.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.split: split_is_callable (surface)."""
import os.path

assert callable(os.path.split)
print("split_is_callable OK")
