# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "create_connection_all_errors_exceptiongroup"
# subject = "socket.create_connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.create_connection: create_connection(..., all_errors=True) to a closed port raises an ExceptionGroup of one OSError per resolved address"""
import socket

probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
probe.bind(("127.0.0.1", 0))
closed_port = probe.getsockname()[1]
probe.close()
try:
    socket.create_connection(("localhost", closed_port), timeout=2, all_errors=True)
    raise AssertionError("connection to closed port should fail")
except ExceptionGroup as eg:
    assert all(isinstance(e, OSError) for e in eg.exceptions), "all sub-errors OSError"
    addrs = socket.getaddrinfo("localhost", closed_port, 0, socket.SOCK_STREAM)
    assert len(addrs) == len(eg.exceptions), "one error per resolved address"
print("create_connection_all_errors_exceptiongroup OK")
