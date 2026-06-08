# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_encodings_map_is_present"
# subject = "mimetypes.encodings_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.encodings_map: api_encodings_map_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "encodings_map")
print("api_encodings_map_is_present OK")
