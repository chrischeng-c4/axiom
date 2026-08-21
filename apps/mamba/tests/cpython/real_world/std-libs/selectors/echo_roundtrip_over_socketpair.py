# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "real_world"
# case = "echo_roundtrip_over_socketpair"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.DefaultSelector: an event-loop-style echo: register both ends of a socketpair, select() to learn when the read end is ready, and verify a byte payload round-trips through the multiplexed selector"""
import selectors
import socket

# A miniature event loop: a client sends a request, the server echoes it back,
# and the client reads the echo — all readiness learned via one selector.
_client, _server = socket.socketpair()
_client.setblocking(False)
_server.setblocking(False)

_payload = b"ping-over-selector"

with selectors.DefaultSelector() as _sel:
    _sel.register(_server, selectors.EVENT_READ, data="server")
    _sel.register(_client, selectors.EVENT_READ, data="client")

    # Client sends the request; drive the loop until the server side is readable.
    _client.sendall(_payload)
    _server_saw = b""
    while not _server_saw:
        for _key, _mask in _sel.select(timeout=2.0):
            if _key.data == "server" and (_mask & selectors.EVENT_READ):
                _server_saw = _key.fileobj.recv(64)
    assert _server_saw == _payload, f"server received {_server_saw!r}"

    # Server echoes; drive the loop until the client side is readable.
    _server.sendall(_server_saw)
    _client_saw = b""
    while not _client_saw:
        for _key, _mask in _sel.select(timeout=2.0):
            if _key.data == "client" and (_mask & selectors.EVENT_READ):
                _client_saw = _key.fileobj.recv(64)
    assert _client_saw == _payload, f"client echo {_client_saw!r}"

_client.close()
_server.close()
print("echo_roundtrip_over_socketpair OK")
