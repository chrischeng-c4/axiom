# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_prio_darwin_bg_is_present"
# subject = "os.PRIO_DARWIN_BG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.PRIO_DARWIN_BG: api_prio_darwin_bg_is_present (surface)."""
import os

assert hasattr(os, "PRIO_DARWIN_BG")
print("api_prio_darwin_bg_is_present OK")
