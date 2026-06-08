# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "lambda_raises"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = "pickle shim serializes unsupported objects to the 'N' (None) sentinel instead of raising PicklingError (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: lambda_raises (errors)."""
import pickle

_raised = False
try:
    pickle.dumps(lambda x: x)
except (pickle.PicklingError, AttributeError):
    _raised = True
assert _raised, "lambda_raises: expected (pickle.PicklingError, AttributeError)"
print("lambda_raises OK")
