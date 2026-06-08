# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_getoutput_is_present"
# subject = "subprocess.getoutput"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.getoutput: api_getoutput_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "getoutput")
print("api_getoutput_is_present OK")
