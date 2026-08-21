# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "expanduser_is_callable"
# subject = "os.path.expanduser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.expanduser: expanduser_is_callable (surface)."""
import os.path

assert callable(os.path.expanduser)
print("expanduser_is_callable OK")
