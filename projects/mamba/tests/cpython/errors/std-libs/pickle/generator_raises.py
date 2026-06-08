# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "generator_raises"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = "pickle shim serializes unsupported objects to the 'N' (None) sentinel instead of raising (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: generator_raises (errors)."""
import pickle

_raised = False
try:
    pickle.dumps(i for i in range(3))
except (TypeError, pickle.PicklingError):
    _raised = True
assert _raised, "generator_raises: expected (TypeError, pickle.PicklingError)"
print("generator_raises OK")
