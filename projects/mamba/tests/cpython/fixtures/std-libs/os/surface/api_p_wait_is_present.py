# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_p_wait_is_present"
# subject = "os.P_WAIT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.P_WAIT: api_p_wait_is_present (surface)."""
import os

assert hasattr(os, "P_WAIT")
print("api_p_wait_is_present OK")
