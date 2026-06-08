# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_isjunction_is_present"
# subject = "os.path.isjunction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.isjunction: api_isjunction_is_present (surface)."""
import os.path

assert hasattr(os.path, "isjunction")
print("api_isjunction_is_present OK")
