# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_urandom_is_present"
# subject = "os.urandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.urandom: api_urandom_is_present (surface)."""
import os

assert hasattr(os, "urandom")
print("api_urandom_is_present OK")
