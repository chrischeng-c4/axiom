# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wnowait_is_present"
# subject = "os.WNOWAIT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WNOWAIT: api_wnowait_is_present (surface)."""
import os

assert hasattr(os, "WNOWAIT")
print("api_wnowait_is_present OK")
