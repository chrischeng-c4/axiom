# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "token_bytes_is_callable"
# subject = "secrets.token_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_bytes: token_bytes_is_callable (surface)."""
import secrets

assert callable(secrets.token_bytes)
print("token_bytes_is_callable OK")
