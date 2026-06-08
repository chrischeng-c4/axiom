# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "token_hex_is_callable"
# subject = "secrets.token_hex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_hex: token_hex_is_callable (surface)."""
import secrets

assert callable(secrets.token_hex)
print("token_hex_is_callable OK")
