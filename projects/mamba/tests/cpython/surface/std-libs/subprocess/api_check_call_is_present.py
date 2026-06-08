# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_check_call_is_present"
# subject = "subprocess.check_call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.check_call: api_check_call_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "check_call")
print("api_check_call_is_present OK")
