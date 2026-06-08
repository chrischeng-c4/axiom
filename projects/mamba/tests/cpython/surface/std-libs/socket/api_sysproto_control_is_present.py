# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_sysproto_control_is_present"
# subject = "socket.SYSPROTO_CONTROL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SYSPROTO_CONTROL: api_sysproto_control_is_present (surface)."""
import socket

assert hasattr(socket, "SYSPROTO_CONTROL")
print("api_sysproto_control_is_present OK")
