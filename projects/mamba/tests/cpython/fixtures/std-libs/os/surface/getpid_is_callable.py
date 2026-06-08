# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "getpid_is_callable"
# subject = "os.getpid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getpid: getpid_is_callable (surface)."""
import os

assert callable(os.getpid)
print("getpid_is_callable OK")
