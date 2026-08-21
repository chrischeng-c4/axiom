# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "pipe_is_int"
# subject = "subprocess.PIPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.PIPE: pipe_is_int (surface)."""
import subprocess

assert type(subprocess.PIPE).__name__ == "int"
print("pipe_is_int OK")
