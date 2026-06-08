# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "select_zero_timeout_returns_list"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.DefaultSelector: select(timeout=0) on a selector with no ready fds returns an empty list without blocking or raising"""
import selectors

with selectors.DefaultSelector() as _sel:
    _result = _sel.select(timeout=0)
    assert isinstance(_result, list), f"select() must return a list, got {type(_result)!r}"
    assert _result == [], f"empty selector select(timeout=0) must be [], got {_result!r}"
print("select_zero_timeout_returns_list OK")
