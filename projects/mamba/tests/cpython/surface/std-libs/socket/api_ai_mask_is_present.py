# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_ai_mask_is_present"
# subject = "socket.AI_MASK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AI_MASK: api_ai_mask_is_present (surface)."""
import socket

assert hasattr(socket, "AI_MASK")
print("api_ai_mask_is_present OK")
