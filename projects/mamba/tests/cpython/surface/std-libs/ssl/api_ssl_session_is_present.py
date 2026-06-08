# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_session_is_present"
# subject = "ssl.SSLSession"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLSession: api_ssl_session_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLSession")
print("api_ssl_session_is_present OK")
