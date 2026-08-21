# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "create_server_binds_loopback_listener"
# subject = "socket.create_server"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.create_server: create_server(('127.0.0.1', 0)) returns an AF_INET SOCK_STREAM socket bound to 127.0.0.1 on a positive ephemeral port; reuse_port toggles SO_REUSEPORT"""
import socket

with socket.create_server(("127.0.0.1", 0)) as sock:
    assert sock.family == socket.AF_INET, f"family = {sock.family!r}"
    assert sock.type == socket.SOCK_STREAM, f"type = {sock.type!r}"
    host, port = sock.getsockname()
    assert host == "127.0.0.1", f"host = {host!r}"
    assert isinstance(port, int) and port > 0, f"port = {port!r}"

if hasattr(socket, "SO_REUSEPORT"):
    with socket.create_server(("127.0.0.1", 0)) as sock:
        assert sock.getsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT) == 0, "default off"
    with socket.create_server(("127.0.0.1", 0), reuse_port=True) as sock:
        assert sock.getsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT) != 0, "reuse_port on"
else:
    raised = False
    try:
        socket.create_server(("127.0.0.1", 0), reuse_port=True)
    except ValueError:
        raised = True
    assert raised, "reuse_port without support should raise ValueError"
print("create_server_binds_loopback_listener OK")
