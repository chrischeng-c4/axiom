# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "devnull_is_int"
# subject = "subprocess.DEVNULL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.DEVNULL: devnull_is_int (surface)."""
import subprocess

assert type(subprocess.DEVNULL).__name__ == "int"
print("devnull_is_int OK")
