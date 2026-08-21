# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "new_class_bad_metaclass_raises"
# subject = "types.new_class"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: new_class_bad_metaclass_raises (errors)."""
import types

_raised = False
try:
    types.new_class('X', (object,), {'metaclass': 'not_a_class'})
except TypeError:
    _raised = True
assert _raised, "new_class_bad_metaclass_raises: expected TypeError"
print("new_class_bad_metaclass_raises OK")
