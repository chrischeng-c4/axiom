# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_mime_types_is_present"
# subject = "mimetypes.MimeTypes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.MimeTypes: api_mime_types_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "MimeTypes")
print("api_mime_types_is_present OK")
