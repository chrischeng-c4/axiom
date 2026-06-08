# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_commonprefix_is_present"
# subject = "os.path.commonprefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.commonprefix: api_commonprefix_is_present (surface)."""
import os.path

assert hasattr(os.path, "commonprefix")
print("api_commonprefix_is_present OK")
