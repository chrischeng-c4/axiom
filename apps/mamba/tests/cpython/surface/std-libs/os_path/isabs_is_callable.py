# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "isabs_is_callable"
# subject = "os.path.isabs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.isabs: isabs_is_callable (surface)."""
import os.path

assert callable(os.path.isabs)
print("isabs_is_callable OK")
