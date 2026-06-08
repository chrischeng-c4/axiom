# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_eai_badflags_is_present"
# subject = "socket.EAI_BADFLAGS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.EAI_BADFLAGS: api_eai_badflags_is_present (surface)."""
import socket

assert hasattr(socket, "EAI_BADFLAGS")
print("api_eai_badflags_is_present OK")
