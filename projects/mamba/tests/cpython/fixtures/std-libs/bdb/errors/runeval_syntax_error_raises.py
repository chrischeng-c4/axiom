# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "errors"
# case = "runeval_syntax_error_raises"
# subject = "bdb.Bdb.runeval"
# kind = "mechanical"
# xfail = "mamba bdb stub: Bdb() is dict-like, no runeval method (#1261)"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
"""bdb.Bdb.runeval: runeval_syntax_error_raises (errors)."""
import bdb

_raised = False
try:
    bdb.Bdb().runeval('def 0bad():')
except SyntaxError:
    _raised = True
assert _raised, "runeval_syntax_error_raises: expected SyntaxError"
print("runeval_syntax_error_raises OK")
