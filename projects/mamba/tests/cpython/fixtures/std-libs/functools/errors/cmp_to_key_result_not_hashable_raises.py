# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "cmp_to_key_result_not_hashable_raises"
# subject = "functools.cmp_to_key"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cmp_to_key: cmp_to_key_result_not_hashable_raises (errors)."""
import functools

_raised = False
try:
    hash(functools.cmp_to_key(lambda x, y: 0)(1))
except TypeError:
    _raised = True
assert _raised, "cmp_to_key_result_not_hashable_raises: expected TypeError"
print("cmp_to_key_result_not_hashable_raises OK")
