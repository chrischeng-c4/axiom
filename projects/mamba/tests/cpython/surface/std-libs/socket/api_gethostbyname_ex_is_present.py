# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_gethostbyname_ex_is_present"
# subject = "socket.gethostbyname_ex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.gethostbyname_ex: api_gethostbyname_ex_is_present (surface)."""
import socket

assert hasattr(socket, "gethostbyname_ex")
print("api_gethostbyname_ex_is_present OK")
