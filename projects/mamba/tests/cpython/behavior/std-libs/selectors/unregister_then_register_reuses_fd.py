# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "unregister_then_register_reuses_fd"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: after unregister(), the same socket can be re-registered with a different events mask"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    _sel.unregister(_s)
    _key = _sel.register(_s, selectors.EVENT_WRITE)
    assert _key.events == selectors.EVENT_WRITE, f"re-registered events = {_key.events!r}"
_s.close()
print("unregister_then_register_reuses_fd OK")
