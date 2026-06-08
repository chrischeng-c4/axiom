# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "stdin_present"
# subject = "sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys: stdin_present (surface)."""
import sys

assert hasattr(sys, "stdin")
print("stdin_present OK")
