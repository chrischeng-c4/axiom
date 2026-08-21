# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "inet_aton_bad_ip_raises"
# subject = "socket.inet_aton"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.inet_aton: inet_aton_bad_ip_raises (errors)."""
import socket

_raised = False
try:
    socket.inet_aton("not.an.ip.address")
except OSError:
    _raised = True
assert _raised, "inet_aton_bad_ip_raises: expected OSError"
print("inet_aton_bad_ip_raises OK")
