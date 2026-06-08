# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "randbelow_negative_raises"
# subject = "secrets.randbelow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbelow: randbelow_negative_raises (errors)."""
import secrets

_raised = False
try:
    secrets.randbelow(-5)
except ValueError:
    _raised = True
assert _raised, "randbelow_negative_raises: expected ValueError"
print("randbelow_negative_raises OK")
