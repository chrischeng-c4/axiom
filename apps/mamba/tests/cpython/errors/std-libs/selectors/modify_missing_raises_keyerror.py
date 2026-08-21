# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "modify_missing_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: modify() on a never-registered socket raises KeyError"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _raised = False
    try:
        _sel.modify(_s, selectors.EVENT_READ)
    except KeyError:
        _raised = True
    assert _raised, "modify of a never-registered socket must raise KeyError"
_s.close()
print("modify_missing_raises_keyerror OK")
