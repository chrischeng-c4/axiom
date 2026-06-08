# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "simplenamespace_positional_arg_raises"
# subject = "types.SimpleNamespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: simplenamespace_positional_arg_raises (errors)."""
import types

_raised = False
try:
    types.SimpleNamespace(1, 2)
except TypeError:
    _raised = True
assert _raised, "simplenamespace_positional_arg_raises: expected TypeError"
print("simplenamespace_positional_arg_raises OK")
