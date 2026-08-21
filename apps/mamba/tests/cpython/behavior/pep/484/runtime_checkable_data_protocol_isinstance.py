# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "runtime_checkable_data_protocol_isinstance"
# subject = "typing.runtime_checkable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: a @runtime_checkable Protocol with a data member supports isinstance by structural presence: a Person with a .name attribute isinstance HasName is True while an Anonymous without it is False"""
from typing import Protocol, runtime_checkable


# A @runtime_checkable Protocol supports isinstance by structural members.
@runtime_checkable
class HasName(Protocol):
    name: str


class Person:
    def __init__(self, name):
        self.name = name


class Anonymous:
    pass


assert isinstance(Person("Ada"), HasName) is True
assert isinstance(Anonymous(), HasName) is False

print("runtime_checkable_data_protocol_isinstance OK")
