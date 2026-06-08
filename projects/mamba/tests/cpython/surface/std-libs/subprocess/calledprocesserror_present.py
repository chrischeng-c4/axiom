# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "calledprocesserror_present"
# subject = "subprocess"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess: calledprocesserror_present (surface)."""
import subprocess

assert hasattr(subprocess, "CalledProcessError")
print("calledprocesserror_present OK")
