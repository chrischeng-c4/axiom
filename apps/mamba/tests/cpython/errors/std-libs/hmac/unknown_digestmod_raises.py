# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "unknown_digestmod_raises"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: unknown_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", b"msg", "no_such_hash")
except ValueError:
    _raised = True
assert _raised, "unknown_digestmod_raises: expected ValueError"
print("unknown_digestmod_raises OK")
