# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "completedprocess_present"
# subject = "subprocess"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess: completedprocess_present (surface)."""
import subprocess

assert hasattr(subprocess, "CompletedProcess")
print("completedprocess_present OK")
