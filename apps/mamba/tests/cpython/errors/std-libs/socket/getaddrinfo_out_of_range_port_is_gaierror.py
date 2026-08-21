# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "getaddrinfo_out_of_range_port_is_gaierror"
# subject = "socket.getaddrinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: an out-of-range port (2**64, 2**63, -(2**63)-1) surfaces as socket.gaierror, never OverflowError; boundary ports 0 and 65535 are accepted"""
import socket

for bad_port in (2**64, 2**63, -(2**63) - 1):
    raised = False
    try:
        socket.getaddrinfo(None, bad_port, type=socket.SOCK_STREAM)
    except socket.gaierror:
        raised = True
    except OverflowError:
        raise AssertionError(f"port {bad_port}: got OverflowError, expected gaierror")
    assert raised, f"port {bad_port} should raise gaierror"

# Boundary ports are accepted.
socket.getaddrinfo(None, 0, type=socket.SOCK_STREAM)
socket.getaddrinfo(None, 65535, type=socket.SOCK_STREAM)
print("getaddrinfo_out_of_range_port_is_gaierror OK")
