# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_sol_socket_is_present"
# subject = "ssl.SOL_SOCKET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SOL_SOCKET: api_sol_socket_is_present (surface)."""
import ssl

assert hasattr(ssl, "SOL_SOCKET")
print("api_sol_socket_is_present OK")
