# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_rtld_nodelete_is_present"
# subject = "os.RTLD_NODELETE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.RTLD_NODELETE: api_rtld_nodelete_is_present (surface)."""
import os

assert hasattr(os, "RTLD_NODELETE")
print("api_rtld_nodelete_is_present OK")
