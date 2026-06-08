# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ai_passive_is_present"
# subject = "socket.AI_PASSIVE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AI_PASSIVE: api_ai_passive_is_present (surface)."""
import socket

assert hasattr(socket, "AI_PASSIVE")
print("api_ai_passive_is_present OK")
