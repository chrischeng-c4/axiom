# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_register_at_fork_is_present"
# subject = "os.register_at_fork"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.register_at_fork: api_register_at_fork_is_present (surface)."""
import os

assert hasattr(os, "register_at_fork")
print("api_register_at_fork_is_present OK")
