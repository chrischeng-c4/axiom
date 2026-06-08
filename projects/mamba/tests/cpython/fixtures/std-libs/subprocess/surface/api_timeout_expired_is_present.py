# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_timeout_expired_is_present"
# subject = "subprocess.TimeoutExpired"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.TimeoutExpired: api_timeout_expired_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "TimeoutExpired")
print("api_timeout_expired_is_present OK")
