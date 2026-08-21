# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "getenv_is_callable"
# subject = "os.getenv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getenv: getenv_is_callable (surface)."""
import os

assert callable(os.getenv)
print("getenv_is_callable OK")
