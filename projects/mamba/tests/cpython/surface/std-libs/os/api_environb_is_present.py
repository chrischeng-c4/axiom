# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_environb_is_present"
# subject = "os.environb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.environb: api_environb_is_present (surface)."""
import os

assert hasattr(os, "environb")
print("api_environb_is_present OK")
