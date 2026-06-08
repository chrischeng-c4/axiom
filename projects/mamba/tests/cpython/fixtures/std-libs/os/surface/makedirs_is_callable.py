# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "makedirs_is_callable"
# subject = "os.makedirs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.makedirs: makedirs_is_callable (surface)."""
import os

assert callable(os.makedirs)
print("makedirs_is_callable OK")
