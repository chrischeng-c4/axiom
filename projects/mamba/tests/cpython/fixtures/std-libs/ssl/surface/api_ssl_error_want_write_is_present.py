# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_error_want_write_is_present"
# subject = "ssl.SSL_ERROR_WANT_WRITE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSL_ERROR_WANT_WRITE: api_ssl_error_want_write_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSL_ERROR_WANT_WRITE")
print("api_ssl_error_want_write_is_present OK")
