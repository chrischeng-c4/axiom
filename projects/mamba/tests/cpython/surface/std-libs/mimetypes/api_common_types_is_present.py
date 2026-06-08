# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_common_types_is_present"
# subject = "mimetypes.common_types"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.common_types: api_common_types_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "common_types")
print("api_common_types_is_present OK")
