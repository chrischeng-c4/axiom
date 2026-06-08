# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "api_scm_rights_is_present"
# subject = "socket.SCM_RIGHTS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""socket.SCM_RIGHTS: api_scm_rights_is_present (surface)."""
import socket

assert hasattr(socket, "SCM_RIGHTS")
print("api_scm_rights_is_present OK")
