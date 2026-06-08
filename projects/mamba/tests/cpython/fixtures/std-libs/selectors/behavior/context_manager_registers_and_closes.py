# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "context_manager_registers_and_closes"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: DefaultSelector used as a context manager registers inside the with-block (get_map length 1) and is usable for the block's duration"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    assert isinstance(_sel, selectors.BaseSelector), "with-target is a selector"
    _sel.register(_s, selectors.EVENT_READ)
    assert len(_sel.get_map()) == 1, "registered inside the context manager"
_s.close()
print("context_manager_registers_and_closes OK")
