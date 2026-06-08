# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_suffix_map_is_present"
# subject = "mimetypes.suffix_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.suffix_map: api_suffix_map_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "suffix_map")
print("api_suffix_map_is_present OK")
