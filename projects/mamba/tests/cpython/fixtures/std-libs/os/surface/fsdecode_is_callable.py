# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "fsdecode_is_callable"
# subject = "os.fsdecode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fsdecode: fsdecode_is_callable (surface)."""
import os

assert callable(os.fsdecode)
print("fsdecode_is_callable OK")
