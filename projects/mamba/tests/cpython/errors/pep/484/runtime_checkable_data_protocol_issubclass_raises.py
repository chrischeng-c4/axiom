# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "runtime_checkable_data_protocol_issubclass_raises"
# subject = "typing.runtime_checkable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: data Protocol rejects issubclass checks."""
from typing import Protocol, runtime_checkable


@runtime_checkable
class HasName(Protocol):
    name: str


class Person:
    name = "Ada"


_raised = False
try:
    issubclass(Person, HasName)
except TypeError:
    _raised = True
assert _raised, "runtime_checkable_data_protocol_issubclass_raises: expected TypeError"

print("runtime_checkable_data_protocol_issubclass_raises OK")
