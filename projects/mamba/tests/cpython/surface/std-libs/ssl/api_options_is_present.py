# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_options_is_present"
# subject = "ssl.Options"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.Options: api_options_is_present (surface)."""
import ssl

assert hasattr(ssl, "Options")
print("api_options_is_present OK")
