# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "errors"
# case = "get_bpbynumber_non_numeric_raises"
# subject = "bdb.Bdb.get_bpbynumber"
# kind = "mechanical"
# xfail = "mamba bdb stub: Bdb() is dict-like, no get_bpbynumber method (#1261)"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
"""bdb.Bdb.get_bpbynumber: get_bpbynumber_non_numeric_raises (errors)."""
import bdb

_raised = False
try:
    bdb.Bdb().get_bpbynumber('not_a_number')
except ValueError:
    _raised = True
assert _raised, "get_bpbynumber_non_numeric_raises: expected ValueError"
print("get_bpbynumber_non_numeric_raises OK")
