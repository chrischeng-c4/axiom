# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "truncated_stream_raises"
# subject = "pickle.loads"
# kind = "mechanical"
# xfail = "pickle shim returns None on bad input, never raises UnpicklingError (src/runtime/stdlib/pickle_mod.rs:324)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: truncated_stream_raises (errors)."""
import pickle

_raised = False
try:
    pickle.loads(b'\x80')
except pickle.UnpicklingError:
    _raised = True
assert _raised, "truncated_stream_raises: expected pickle.UnpicklingError"
print("truncated_stream_raises OK")
