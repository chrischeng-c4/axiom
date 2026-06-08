# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "token_bytes_negative_raises"
# subject = "secrets.token_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_bytes: token_bytes_negative_raises (errors)."""
import secrets

_raised = False
try:
    secrets.token_bytes(-1)
except ValueError:
    _raised = True
assert _raised, "token_bytes_negative_raises: expected ValueError"
print("token_bytes_negative_raises OK")
