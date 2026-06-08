# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "empty_digestmod_raises"
# subject = "hmac.HMAC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC: empty_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.HMAC(b"key", msg=b"msg", digestmod="")
except TypeError:
    _raised = True
assert _raised, "empty_digestmod_raises: expected TypeError"
print("empty_digestmod_raises OK")
