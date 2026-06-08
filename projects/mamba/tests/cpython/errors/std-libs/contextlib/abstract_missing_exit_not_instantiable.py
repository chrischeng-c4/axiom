# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "abstract_missing_exit_not_instantiable"
# subject = "contextlib.AbstractContextManager"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.AbstractContextManager: abstract_missing_exit_not_instantiable (errors)."""
import contextlib

_raised = False
try:
    type('MissingExit', (contextlib.AbstractContextManager,), {})()
except TypeError:
    _raised = True
assert _raised, "abstract_missing_exit_not_instantiable: expected TypeError"
print("abstract_missing_exit_not_instantiable OK")
