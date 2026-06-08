# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "stdout_is_int"
# subject = "subprocess.STDOUT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.STDOUT: stdout_is_int (surface)."""
import subprocess

assert type(subprocess.STDOUT).__name__ == "int"
print("stdout_is_int OK")
