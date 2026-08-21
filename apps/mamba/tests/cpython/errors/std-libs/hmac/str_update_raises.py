# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "str_update_raises"
# subject = "hmac.HMAC.update"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC.update: str_update_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", digestmod="sha256").update("not bytes")
except TypeError:
    _raised = True
assert _raised, "str_update_raises: expected TypeError"
print("str_update_raises OK")
