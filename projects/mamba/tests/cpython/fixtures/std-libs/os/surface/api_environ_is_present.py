# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_environ_is_present"
# subject = "os.environ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.environ: api_environ_is_present (surface)."""
import os

assert hasattr(os, "environ")
print("api_environ_is_present OK")
