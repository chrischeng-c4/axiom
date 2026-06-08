# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_socket_error_is_present"
# subject = "ssl.socket_error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.socket_error: api_socket_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "socket_error")
print("api_socket_error_is_present OK")
