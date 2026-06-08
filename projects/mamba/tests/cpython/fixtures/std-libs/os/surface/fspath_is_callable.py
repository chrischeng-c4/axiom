# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "fspath_is_callable"
# subject = "os.fspath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fspath: fspath_is_callable (surface)."""
import os

assert callable(os.fspath)
print("fspath_is_callable OK")
