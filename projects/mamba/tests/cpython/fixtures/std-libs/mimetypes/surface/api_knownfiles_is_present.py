# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_knownfiles_is_present"
# subject = "mimetypes.knownfiles"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.knownfiles: api_knownfiles_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "knownfiles")
print("api_knownfiles_is_present OK")
