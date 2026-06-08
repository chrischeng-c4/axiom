# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_init_is_present"
# subject = "mimetypes.init"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.init: api_init_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "init")
print("api_init_is_present OK")
