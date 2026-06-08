# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "surface"
# case = "api_getstatusoutput_is_present"
# subject = "subprocess.getstatusoutput"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""subprocess.getstatusoutput: api_getstatusoutput_is_present (surface)."""
import subprocess

assert hasattr(subprocess, "getstatusoutput")
print("api_getstatusoutput_is_present OK")
