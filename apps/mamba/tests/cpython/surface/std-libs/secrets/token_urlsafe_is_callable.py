# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "token_urlsafe_is_callable"
# subject = "secrets.token_urlsafe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_urlsafe: token_urlsafe_is_callable (surface)."""
import secrets

assert callable(secrets.token_urlsafe)
print("token_urlsafe_is_callable OK")
