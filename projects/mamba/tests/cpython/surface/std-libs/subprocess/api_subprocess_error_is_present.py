# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_subprocess_error_is_present"
# subject = "subprocess.SubprocessError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.SubprocessError: api_subprocess_error_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "SubprocessError")
print("api_subprocess_error_is_present OK")
