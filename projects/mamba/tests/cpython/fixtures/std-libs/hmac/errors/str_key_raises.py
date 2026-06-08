# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "str_key_raises"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: str_key_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new("not bytes", b"msg", "sha256")
except TypeError:
    _raised = True
assert _raised, "str_key_raises: expected TypeError"
print("str_key_raises OK")
