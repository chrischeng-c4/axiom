# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_inited_is_present"
# subject = "mimetypes.inited"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.inited: api_inited_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "inited")
print("api_inited_is_present OK")
