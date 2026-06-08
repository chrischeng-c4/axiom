# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "randbelow_zero_raises"
# subject = "secrets.randbelow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbelow: randbelow_zero_raises (errors)."""
import secrets

_raised = False
try:
    secrets.randbelow(0)
except ValueError:
    _raised = True
assert _raised, "randbelow_zero_raises: expected ValueError"
print("randbelow_zero_raises OK")
