# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "get_map_returns_registered_keys"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: get_map() exposes the registered fd->SelectorKey mapping; the registered socket's fd is present and maps back to its key"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _key = _sel.register(_s, selectors.EVENT_READ, data="payload")
    _map = _sel.get_map()
    assert _key.fd in _map, "registered fd must be a key in get_map()"
    assert _map[_key.fd] is _key, "get_map() must map the fd back to its SelectorKey"
    assert _map[_key.fd].data == "payload", "mapped key carries the registered data"
_s.close()
print("get_map_returns_registered_keys OK")
