# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ai_numerichost_is_present"
# subject = "socket.AI_NUMERICHOST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AI_NUMERICHOST: api_ai_numerichost_is_present (surface)."""
import socket

assert hasattr(socket, "AI_NUMERICHOST")
print("api_ai_numerichost_is_present OK")
