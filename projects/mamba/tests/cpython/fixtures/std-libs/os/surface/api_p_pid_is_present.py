# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_p_pid_is_present"
# subject = "os.P_PID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.P_PID: api_p_pid_is_present (surface)."""
import os

assert hasattr(os, "P_PID")
print("api_p_pid_is_present OK")
