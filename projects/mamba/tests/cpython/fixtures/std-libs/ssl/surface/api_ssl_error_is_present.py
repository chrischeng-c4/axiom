# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_error_is_present"
# subject = "ssl.SSLError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLError: api_ssl_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLError")
print("api_ssl_error_is_present OK")
