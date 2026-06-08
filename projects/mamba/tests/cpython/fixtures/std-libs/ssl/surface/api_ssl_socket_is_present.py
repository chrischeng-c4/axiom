# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_socket_is_present"
# subject = "ssl.SSLSocket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLSocket: api_ssl_socket_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLSocket")
print("api_ssl_socket_is_present OK")
