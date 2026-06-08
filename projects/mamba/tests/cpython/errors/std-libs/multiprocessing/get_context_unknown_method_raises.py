# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "get_context_unknown_method_raises"
# subject = "multiprocessing.get_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.get_context: get_context_unknown_method_raises (errors)."""
import multiprocessing

_raised = False
try:
    multiprocessing.get_context('no_such_method')
except ValueError:
    _raised = True
assert _raised, "get_context_unknown_method_raises: expected ValueError"
print("get_context_unknown_method_raises OK")
