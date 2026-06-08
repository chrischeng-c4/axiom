# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "randbelow_is_callable"
# subject = "secrets.randbelow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.randbelow: randbelow_is_callable (surface)."""
import secrets

assert callable(secrets.randbelow)
print("randbelow_is_callable OK")
