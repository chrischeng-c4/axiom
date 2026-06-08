# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_af_sna_is_present"
# subject = "socket.AF_SNA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.AF_SNA: api_af_sna_is_present (surface)."""
import socket

assert hasattr(socket, "AF_SNA")
print("api_af_sna_is_present OK")
