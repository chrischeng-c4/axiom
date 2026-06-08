# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_pipe_is_present"
# subject = "os.pipe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.pipe: api_pipe_is_present (surface)."""
import os

assert hasattr(os, "pipe")
print("api_pipe_is_present OK")
