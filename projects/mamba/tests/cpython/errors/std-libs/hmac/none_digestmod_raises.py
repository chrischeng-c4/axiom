# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "none_digestmod_raises"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: none_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", b"msg", None)
except TypeError:
    _raised = True
assert _raised, "none_digestmod_raises: expected TypeError"
print("none_digestmod_raises OK")
