# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_eai_protocol_is_present"
# subject = "socket.EAI_PROTOCOL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.EAI_PROTOCOL: api_eai_protocol_is_present (surface)."""
import socket

assert hasattr(socket, "EAI_PROTOCOL")
print("api_eai_protocol_is_present OK")
