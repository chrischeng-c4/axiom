# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "timeoutexpired_present"
# subject = "subprocess"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess: timeoutexpired_present (surface)."""
import subprocess

assert hasattr(subprocess, "TimeoutExpired")
print("timeoutexpired_present OK")
