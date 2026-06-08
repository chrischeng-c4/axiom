# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "api_guess_extension_is_present"
# subject = "mimetypes.guess_extension"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""mimetypes.guess_extension: api_guess_extension_is_present (surface)."""
import mimetypes

assert hasattr(mimetypes, "guess_extension")
print("api_guess_extension_is_present OK")
