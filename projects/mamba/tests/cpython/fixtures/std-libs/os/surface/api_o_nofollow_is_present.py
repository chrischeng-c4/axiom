# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_o_nofollow_is_present"
# subject = "os.O_NOFOLLOW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.O_NOFOLLOW: api_o_nofollow_is_present (surface)."""
import os

assert hasattr(os, "O_NOFOLLOW")
print("api_o_nofollow_is_present OK")
