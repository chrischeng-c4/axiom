# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "runtime_checkable_method_protocol_isinstance"
# subject = "typing.runtime_checkable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: a @runtime_checkable method Protocol matches by attribute presence: a File with a close() method isinstance Closeable is True while the int 42 is False"""
from typing import Protocol, runtime_checkable


# A method-based runtime_checkable Protocol matches by attribute presence.
@runtime_checkable
class Closeable(Protocol):
    def close(self) -> None: ...


class File:
    def close(self):
        pass


assert isinstance(File(), Closeable) is True
assert isinstance(42, Closeable) is False

print("runtime_checkable_method_protocol_isinstance OK")
