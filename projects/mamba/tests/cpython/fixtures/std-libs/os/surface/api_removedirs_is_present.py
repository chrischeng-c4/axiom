# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_removedirs_is_present"
# subject = "os.removedirs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.removedirs: api_removedirs_is_present (surface)."""
import os

assert hasattr(os, "removedirs")
print("api_removedirs_is_present OK")
