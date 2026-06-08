# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "stderr_present"
# subject = "sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys: stderr_present (surface)."""
import sys

assert hasattr(sys, "stderr")
print("stderr_present OK")
