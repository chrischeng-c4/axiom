# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "mixed_type_compare_raises"
# subject = "hmac.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.compare_digest: mixed_type_compare_raises (errors)."""
import hmac

_raised = False
try:
    hmac.compare_digest("string", b"bytes")
except TypeError:
    _raised = True
assert _raised, "mixed_type_compare_raises: expected TypeError"
print("mixed_type_compare_raises OK")
