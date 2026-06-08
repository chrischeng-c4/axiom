# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_o_rdwr_is_present"
# subject = "os.O_RDWR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.O_RDWR: api_o_rdwr_is_present (surface)."""
import os

assert hasattr(os, "O_RDWR")
print("api_o_rdwr_is_present OK")
