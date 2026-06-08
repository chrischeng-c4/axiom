# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_completed_process_is_present"
# subject = "subprocess.CompletedProcess"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.CompletedProcess: api_completed_process_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "CompletedProcess")
print("api_completed_process_is_present OK")
