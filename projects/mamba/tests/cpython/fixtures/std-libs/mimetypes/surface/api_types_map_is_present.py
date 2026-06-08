# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_types_map_is_present"
# subject = "mimetypes.types_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.types_map: api_types_map_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "types_map")
print("api_types_map_is_present OK")
