# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "token_urlsafe_negative_raises"
# subject = "secrets.token_urlsafe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_urlsafe: token_urlsafe_negative_raises (errors)."""
import secrets

_raised = False
try:
    secrets.token_urlsafe(-1)
except ValueError:
    _raised = True
assert _raised, "token_urlsafe_negative_raises: expected ValueError"
print("token_urlsafe_negative_raises OK")
