# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "mutable_list_default_raises"
# subject = "dataclasses.dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: mutable_list_default_raises (errors)."""
import dataclasses
from typing import List

_raised = False
try:
    dataclasses.dataclass(type('Bad', (), {'__annotations__': {'items': list}, 'items': []}))
except ValueError:
    _raised = True
assert _raised, "mutable_list_default_raises: expected ValueError"
print("mutable_list_default_raises OK")
