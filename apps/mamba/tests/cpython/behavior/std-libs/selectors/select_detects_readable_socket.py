# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "select_detects_readable_socket"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: select() reports a listening server socket as readable once a client connects; the ready key's fileobj is the server socket and the mask carries EVENT_READ"""
import selectors
import socket
import threading
import time

_srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
_srv.bind(("127.0.0.1", 0))
_port = _srv.getsockname()[1]
_srv.listen(1)

_holder = [None]

def _connect_after_delay():
    time.sleep(0.05)
    _c = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    _c.connect(("127.0.0.1", _port))
    _holder[0] = _c

_t = threading.Thread(target=_connect_after_delay)
_t.start()

with selectors.DefaultSelector() as _sel:
    _sel.register(_srv, selectors.EVENT_READ)
    _ready = _sel.select(timeout=2.0)
    assert len(_ready) == 1, f"exactly one ready event expected, got {len(_ready)}"
    _key, _mask = _ready[0]
    assert _mask & selectors.EVENT_READ, "ready mask must carry EVENT_READ"
    assert _key.fileobj is _srv, "ready key's fileobj must be the server socket"

_t.join()
if _holder[0] is not None:
    _holder[0].close()
_srv.close()
print("select_detects_readable_socket OK")
