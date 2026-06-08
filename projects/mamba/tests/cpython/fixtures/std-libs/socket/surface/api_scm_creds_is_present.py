# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_scm_creds_is_present"
# subject = "socket.SCM_CREDS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SCM_CREDS: api_scm_creds_is_present (surface)."""
import socket

assert hasattr(socket, "SCM_CREDS")
print("api_scm_creds_is_present OK")
