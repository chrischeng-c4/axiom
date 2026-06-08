# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_want_read_error_is_present"
# subject = "ssl.SSLWantReadError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLWantReadError: api_ssl_want_read_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLWantReadError")
print("api_ssl_want_read_error_is_present OK")
