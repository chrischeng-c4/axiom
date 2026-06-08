# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "proxy_int_raises"
# subject = "weakref.proxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy_int_raises (errors)."""
import weakref

_raised = False
try:
    weakref.proxy(42)
except TypeError:
    _raised = True
assert _raised, "proxy_int_raises: expected TypeError"
print("proxy_int_raises OK")
