# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_check_output_is_present"
# subject = "subprocess.check_output"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.check_output: api_check_output_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "check_output")
print("api_check_output_is_present OK")
