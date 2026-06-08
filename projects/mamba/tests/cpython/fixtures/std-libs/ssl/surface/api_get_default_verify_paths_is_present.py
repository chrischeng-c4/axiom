# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_get_default_verify_paths_is_present"
# subject = "ssl.get_default_verify_paths"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.get_default_verify_paths: api_get_default_verify_paths_is_present (surface)."""
import ssl

assert hasattr(ssl, "get_default_verify_paths")
print("api_get_default_verify_paths_is_present OK")
