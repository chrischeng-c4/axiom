# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "choice_empty_sequence_raises"
# subject = "secrets.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.choice: choice_empty_sequence_raises (errors)."""
import secrets

_raised = False
try:
    secrets.choice([])
except IndexError:
    _raised = True
assert _raised, "choice_empty_sequence_raises: expected IndexError"
print("choice_empty_sequence_raises OK")
