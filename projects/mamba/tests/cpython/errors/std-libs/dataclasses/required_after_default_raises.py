# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "required_after_default_raises"
# subject = "dataclasses.dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: required_after_default_raises (errors)."""
import dataclasses

_raised = False
try:
    dataclasses.dataclass(type('WrongOrder', (), {'__annotations__': {'a': int, 'b': int}, 'a': 1}))
except TypeError:
    _raised = True
assert _raised, "required_after_default_raises: expected TypeError"
print("required_after_default_raises OK")
