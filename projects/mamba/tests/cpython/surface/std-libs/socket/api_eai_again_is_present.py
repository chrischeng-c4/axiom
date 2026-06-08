# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_eai_again_is_present"
# subject = "socket.EAI_AGAIN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.EAI_AGAIN: api_eai_again_is_present (surface)."""
import socket

assert hasattr(socket, "EAI_AGAIN")
print("api_eai_again_is_present OK")
