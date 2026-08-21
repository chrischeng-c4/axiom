# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "stdout_present"
# subject = "sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys: stdout_present (surface)."""
import sys

assert hasattr(sys, "stdout")
print("stdout_present OK")
