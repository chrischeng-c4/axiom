# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "errors"
# case = "tok_name_unknown_key_raises_keyerror"
# subject = "token.tok_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.tok_name: tok_name_unknown_key_raises_keyerror (errors)."""
import token

_raised = False
try:
    token.tok_name[99999]
except KeyError:
    _raised = True
assert _raised, "tok_name_unknown_key_raises_keyerror: expected KeyError"
print("tok_name_unknown_key_raises_keyerror OK")
