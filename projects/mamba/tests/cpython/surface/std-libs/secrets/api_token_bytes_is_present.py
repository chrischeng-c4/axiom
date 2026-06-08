# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "api_token_bytes_is_present"
# subject = "secrets.token_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""secrets.token_bytes: api_token_bytes_is_present (surface)."""
import secrets

assert hasattr(secrets, "token_bytes")
print("api_token_bytes_is_present OK")
