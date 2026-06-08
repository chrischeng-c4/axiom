# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "compare_digest_bytes_str_raises"
# subject = "secrets.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest_bytes_str_raises (errors)."""
import secrets

_raised = False
try:
    secrets.compare_digest(b"abc", "abc")
except TypeError:
    _raised = True
assert _raised, "compare_digest_bytes_str_raises: expected TypeError"
print("compare_digest_bytes_str_raises OK")
