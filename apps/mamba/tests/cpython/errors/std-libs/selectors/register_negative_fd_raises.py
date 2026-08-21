# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "errors"
# case = "register_negative_fd_raises"
# subject = "selectors.DefaultSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: register_negative_fd_raises (errors)."""
import selectors

_raised = False
try:
    selectors.DefaultSelector().register(-10, selectors.EVENT_READ)
except ValueError:
    _raised = True
assert _raised, "register_negative_fd_raises: expected ValueError"
print("register_negative_fd_raises OK")
