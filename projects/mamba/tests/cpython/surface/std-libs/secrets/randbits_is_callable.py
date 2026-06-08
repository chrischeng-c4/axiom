# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "randbits_is_callable"
# subject = "secrets.randbits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.randbits: randbits_is_callable (surface)."""
import secrets

assert callable(secrets.randbits)
print("randbits_is_callable OK")
