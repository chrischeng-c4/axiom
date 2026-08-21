# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "unregister_missing_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: unregistering a never-registered socket raises KeyError"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _raised = False
    try:
        _sel.unregister(_s)
    except KeyError:
        _raised = True
    assert _raised, "unregister of a never-registered socket must raise KeyError"
_s.close()
print("unregister_missing_raises_keyerror OK")
