# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wtermsig_is_present"
# subject = "os.WTERMSIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WTERMSIG: api_wtermsig_is_present (surface)."""
import os

assert hasattr(os, "WTERMSIG")
print("api_wtermsig_is_present OK")
