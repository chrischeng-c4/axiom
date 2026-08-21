# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "errors"
# case = "build_opener_non_handler_raises"
# subject = "urllib.request.build_opener"
# kind = "mechanical"
# xfail = "urllib.request unimplemented on mamba: build_opener is a stub that does not raise on a non-handler arg (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.build_opener: build_opener_non_handler_raises (errors)."""
from urllib.request import build_opener

_raised = False
try:
    build_opener('not_a_handler')
except TypeError:
    _raised = True
assert _raised, "build_opener_non_handler_raises: expected TypeError"
print("build_opener_non_handler_raises OK")
