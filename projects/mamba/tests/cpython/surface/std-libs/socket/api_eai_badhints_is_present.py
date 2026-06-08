# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_eai_badhints_is_present"
# subject = "socket.EAI_BADHINTS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.EAI_BADHINTS: api_eai_badhints_is_present (surface)."""
import socket

assert hasattr(socket, "EAI_BADHINTS")
print("api_eai_badhints_is_present OK")
