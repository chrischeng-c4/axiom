# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "compare_digest_str_bytes_raises"
# subject = "secrets.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest_str_bytes_raises (errors)."""
import secrets

_raised = False
try:
    secrets.compare_digest("abc", b"abc")
except TypeError:
    _raised = True
assert _raised, "compare_digest_str_bytes_raises: expected TypeError"
print("compare_digest_str_bytes_raises OK")
