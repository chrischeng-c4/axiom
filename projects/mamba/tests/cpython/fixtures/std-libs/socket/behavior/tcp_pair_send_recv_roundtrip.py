# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "tcp_pair_send_recv_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a loopback TCP socket pair (server thread + client) exchanges ping/pong: the server receives b'ping' and the client receives the b'pong' reply"""
import socket
import threading

_srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
_srv.bind(("127.0.0.1", 0))
_port = _srv.getsockname()[1]
_srv.listen(1)

_received = []


def _server_thread():
    _conn, _ = _srv.accept()
    _data = _conn.recv(1024)
    _received.append(_data)
    _conn.sendall(b"pong")
    _conn.close()


_t = threading.Thread(target=_server_thread)
_t.start()

_cli = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_cli.connect(("127.0.0.1", _port))
_cli.sendall(b"ping")
_response = _cli.recv(1024)
_cli.close()
_t.join()
_srv.close()

assert _received == [b"ping"], f"server received: {_received!r}"
assert _response == b"pong", f"client received: {_response!r}"
print("tcp_pair_send_recv_roundtrip OK")
