# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "value_bad_typecode_raises"
# subject = "multiprocessing.Value"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Value: value_bad_typecode_raises (errors)."""
import multiprocessing

_raised = False
try:
    multiprocessing.Value('not_a_typecode')
except (AttributeError, TypeError):
    _raised = True
assert _raised, "value_bad_typecode_raises: expected (AttributeError, TypeError)"
print("value_bad_typecode_raises OK")
