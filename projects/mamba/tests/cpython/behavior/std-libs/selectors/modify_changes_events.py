# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "modify_changes_events"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: modify() updates the registered events mask in place and returns the updated SelectorKey, observable via get_key()"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    _key2 = _sel.modify(_s, selectors.EVENT_READ | selectors.EVENT_WRITE)
    assert _key2.events == (selectors.EVENT_READ | selectors.EVENT_WRITE), f"modified events = {_key2.events!r}"
    assert _sel.get_key(_s).events == (selectors.EVENT_READ | selectors.EVENT_WRITE), "get_key reflects the modify"
_s.close()
print("modify_changes_events OK")
