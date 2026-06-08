# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "join_is_callable"
# subject = "os.path.join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.join: join_is_callable (surface)."""
import os.path

assert callable(os.path.join)
print("join_is_callable OK")
