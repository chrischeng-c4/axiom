# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_called_process_error_is_present"
# subject = "subprocess.CalledProcessError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.CalledProcessError: api_called_process_error_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "CalledProcessError")
print("api_called_process_error_is_present OK")
