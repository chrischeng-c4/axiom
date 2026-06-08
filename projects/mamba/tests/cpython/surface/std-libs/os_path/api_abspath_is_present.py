# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_abspath_is_present"
# subject = "os.path.abspath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.abspath: api_abspath_is_present (surface)."""
import os.path

assert hasattr(os.path, "abspath")
print("api_abspath_is_present OK")
