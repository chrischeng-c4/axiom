# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_run_is_present"
# subject = "subprocess.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.run: api_run_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "run")
print("api_run_is_present OK")
