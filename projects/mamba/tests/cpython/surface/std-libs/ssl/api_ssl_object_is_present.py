# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_object_is_present"
# subject = "ssl.SSLObject"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLObject: api_ssl_object_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLObject")
print("api_ssl_object_is_present OK")
