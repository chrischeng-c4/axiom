# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "register_duplicate_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: registering the same socket twice on one DefaultSelector raises KeyError on the second register"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    _raised = False
    try:
        _sel.register(_s, selectors.EVENT_WRITE)
    except KeyError:
        _raised = True
    assert _raised, "second register of same socket must raise KeyError"
_s.close()
print("register_duplicate_raises_keyerror OK")
