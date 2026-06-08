# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_context_is_present"
# subject = "ssl.SSLContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLContext: api_ssl_context_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLContext")
print("api_ssl_context_is_present OK")
