# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_base64_is_present"
# subject = "ssl.base64"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.base64: api_base64_is_present (surface)."""
import ssl

assert hasattr(ssl, "base64")
print("api_base64_is_present OK")
