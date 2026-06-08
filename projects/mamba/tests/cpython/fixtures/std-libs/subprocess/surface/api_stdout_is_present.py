# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_stdout_is_present"
# subject = "subprocess.STDOUT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.STDOUT: api_stdout_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "STDOUT")
print("api_stdout_is_present OK")
