# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_inaddr_allhosts_group_is_present"
# subject = "socket.INADDR_ALLHOSTS_GROUP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.INADDR_ALLHOSTS_GROUP: api_inaddr_allhosts_group_is_present (surface)."""
import socket

assert hasattr(socket, "INADDR_ALLHOSTS_GROUP")
print("api_inaddr_allhosts_group_is_present OK")
