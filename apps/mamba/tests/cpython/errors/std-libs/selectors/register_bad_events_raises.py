# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "register_bad_events_raises"
# subject = "selectors.DefaultSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: register_bad_events_raises (errors)."""
import selectors
import socket

_raised = False
try:
    selectors.DefaultSelector().register(socket.socket(), 999, None)
except ValueError:
    _raised = True
assert _raised, "register_bad_events_raises: expected ValueError"
print("register_bad_events_raises OK")
