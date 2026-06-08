# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "unregister_clears_map"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: unregister() removes the fd so get_map() is empty afterward"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    assert len(_sel.get_map()) == 1, "map has one entry after register"
    _sel.unregister(_s)
    assert len(_sel.get_map()) == 0, f"map must be empty after unregister, got {len(_sel.get_map())}"
_s.close()
print("unregister_clears_map OK")
