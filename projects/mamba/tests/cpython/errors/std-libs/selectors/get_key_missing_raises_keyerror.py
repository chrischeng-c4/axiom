# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "get_key_missing_raises_keyerror"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: get_key() on a never-registered file object raises KeyError"""
import selectors

with selectors.DefaultSelector() as _sel:
    _raised = False
    try:
        _sel.get_key(99999)
    except KeyError:
        _raised = True
    assert _raised, "get_key of a never-registered file object must raise KeyError"
print("get_key_missing_raises_keyerror OK")
