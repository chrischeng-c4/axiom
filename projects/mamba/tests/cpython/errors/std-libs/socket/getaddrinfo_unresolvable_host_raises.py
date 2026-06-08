# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "getaddrinfo_unresolvable_host_raises"
# subject = "socket.getaddrinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: getaddrinfo_unresolvable_host_raises (errors)."""
import socket

_raised = False
try:
    socket.getaddrinfo("definitely_not_a_real_host_xyzzy.invalid", 80)
except socket.gaierror:
    _raised = True
assert _raised, "getaddrinfo_unresolvable_host_raises: expected socket.gaierror"
print("getaddrinfo_unresolvable_host_raises OK")
