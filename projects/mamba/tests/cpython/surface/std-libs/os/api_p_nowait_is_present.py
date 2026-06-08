# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_p_nowait_is_present"
# subject = "os.P_NOWAIT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.P_NOWAIT: api_p_nowait_is_present (surface)."""
import os

assert hasattr(os, "P_NOWAIT")
print("api_p_nowait_is_present OK")
