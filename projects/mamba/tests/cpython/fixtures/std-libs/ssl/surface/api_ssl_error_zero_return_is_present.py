# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_error_zero_return_is_present"
# subject = "ssl.SSL_ERROR_ZERO_RETURN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSL_ERROR_ZERO_RETURN: api_ssl_error_zero_return_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSL_ERROR_ZERO_RETURN")
print("api_ssl_error_zero_return_is_present OK")
