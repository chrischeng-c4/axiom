# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "surface"
# case = "api_clear_extension_cache_is_present"
# subject = "copyreg.clear_extension_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copyreg.clear_extension_cache: api_clear_extension_cache_is_present (surface)."""
import copyreg

assert hasattr(copyreg, "clear_extension_cache")
print("api_clear_extension_cache_is_present OK")
