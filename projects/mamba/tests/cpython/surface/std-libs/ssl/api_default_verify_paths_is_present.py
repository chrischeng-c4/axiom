# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_default_verify_paths_is_present"
# subject = "ssl.DefaultVerifyPaths"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.DefaultVerifyPaths: api_default_verify_paths_is_present (surface)."""
import ssl

assert hasattr(ssl, "DefaultVerifyPaths")
print("api_default_verify_paths_is_present OK")
