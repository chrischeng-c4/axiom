# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "abspath_is_callable"
# subject = "os.path.abspath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.abspath: abspath_is_callable (surface)."""
import os.path

assert callable(os.path.abspath)
print("abspath_is_callable OK")
