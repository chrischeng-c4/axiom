# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_read_mime_types_is_present"
# subject = "mimetypes.read_mime_types"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.read_mime_types: api_read_mime_types_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "read_mime_types")
print("api_read_mime_types_is_present OK")
