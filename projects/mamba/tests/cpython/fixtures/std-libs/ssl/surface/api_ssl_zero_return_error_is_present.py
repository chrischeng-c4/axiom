# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_zero_return_error_is_present"
# subject = "ssl.SSLZeroReturnError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLZeroReturnError: api_ssl_zero_return_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLZeroReturnError")
print("api_ssl_zero_return_error_is_present OK")
