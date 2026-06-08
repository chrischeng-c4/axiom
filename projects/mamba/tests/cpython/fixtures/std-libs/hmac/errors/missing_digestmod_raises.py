# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "missing_digestmod_raises"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: missing_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", b"msg")
except TypeError:
    _raised = True
assert _raised, "missing_digestmod_raises: expected TypeError"
print("missing_digestmod_raises OK")
