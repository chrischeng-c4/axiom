# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "samefile_is_callable"
# subject = "os.path.samefile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.samefile: samefile_is_callable (surface)."""
import os.path

assert callable(os.path.samefile)
print("samefile_is_callable OK")
