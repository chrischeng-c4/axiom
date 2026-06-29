# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "runtime_checkable_method_protocol_issubclass"
# subject = "typing.runtime_checkable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: method Protocol supports issubclass structural checks."""
from typing import Protocol, runtime_checkable


@runtime_checkable
class Closeable(Protocol):
    def close(self) -> None: ...


class File:
    def close(self):
        pass


class Anonymous:
    pass


assert issubclass(File, Closeable) is True
assert issubclass(Anonymous, Closeable) is False

print("runtime_checkable_method_protocol_issubclass OK")
