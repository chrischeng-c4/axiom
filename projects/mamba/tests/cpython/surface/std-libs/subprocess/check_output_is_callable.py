# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "check_output_is_callable"
# subject = "subprocess.check_output"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""subprocess.check_output: check_output_is_callable (surface)."""
import subprocess

assert callable(subprocess.check_output)
print("check_output_is_callable OK")
