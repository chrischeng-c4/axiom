# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "select_detects_writable_socket"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: a freshly connected socketpair endpoint registered for EVENT_WRITE is reported writable by select() with EVENT_WRITE in the mask"""
import selectors
import socket

_a, _b = socket.socketpair()
with selectors.DefaultSelector() as _sel:
    _sel.register(_a, selectors.EVENT_WRITE)
    _ready = _sel.select(timeout=0.5)
    assert len(_ready) >= 1, f"connected endpoint should be writable, got {_ready!r}"
    _key, _mask = _ready[0]
    assert _mask & selectors.EVENT_WRITE, "ready mask must carry EVENT_WRITE"
    assert _key.fileobj is _a, "ready key's fileobj must be the registered socket"
_a.close()
_b.close()
print("select_detects_writable_socket OK")
