# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "errors"
# case = "bad_protocol_raises"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = "pickle shim ignores the protocol kwarg and never validates it (src/runtime/stdlib/pickle_mod.rs:318)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.dumps: bad_protocol_raises (errors)."""
import pickle

_raised = False
try:
    pickle.dumps('hi', protocol=99)
except ValueError:
    _raised = True
assert _raised, "bad_protocol_raises: expected ValueError"
print("bad_protocol_raises OK")
