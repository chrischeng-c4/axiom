# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "non_callable_default_factory_raises"
# subject = "dataclasses.field"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.field: non_callable_default_factory_raises (errors)."""
import dataclasses

_raised = False
try:
    dataclasses.dataclass(type('BadFactory', (), {'__annotations__': {'items': list}, 'items': dataclasses.field(default_factory=42)}))()
except TypeError:
    _raised = True
assert _raised, "non_callable_default_factory_raises: expected TypeError"
print("non_callable_default_factory_raises OK")
