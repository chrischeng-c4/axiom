# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "getcwd_is_callable"
# subject = "os.getcwd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getcwd: getcwd_is_callable (surface)."""
import os

assert callable(os.getcwd)
print("getcwd_is_callable OK")
