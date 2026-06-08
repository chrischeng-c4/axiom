# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "fsencode_is_callable"
# subject = "os.fsencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fsencode: fsencode_is_callable (surface)."""
import os

assert callable(os.fsencode)
print("fsencode_is_callable OK")
