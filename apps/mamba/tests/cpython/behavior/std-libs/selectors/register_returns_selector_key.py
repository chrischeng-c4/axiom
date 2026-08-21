# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "register_returns_selector_key"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: register() returns a SelectorKey whose fileobj is the socket, fd is an int, events is the requested mask, and data is the passed-in object"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _key = _sel.register(_s, selectors.EVENT_READ | selectors.EVENT_WRITE, data=42)
    assert isinstance(_key, selectors.SelectorKey), f"register() must return SelectorKey, got {type(_key)!r}"
    assert _key.fileobj is _s, "key.fileobj must be the registered socket"
    assert isinstance(_key.fd, int), f"key.fd must be int, got {type(_key.fd)!r}"
    assert _key.events == (selectors.EVENT_READ | selectors.EVENT_WRITE), f"key.events = {_key.events!r}"
    assert _key.data == 42, f"key.data = {_key.data!r}"
_s.close()
print("register_returns_selector_key OK")
