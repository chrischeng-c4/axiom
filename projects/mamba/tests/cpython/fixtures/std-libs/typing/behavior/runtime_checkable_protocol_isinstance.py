# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "runtime_checkable_protocol_isinstance"
# subject = "typing.runtime_checkable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.runtime_checkable: a @runtime_checkable Protocol declaring __len__ supports isinstance: a list satisfies it (True) while an int does not (False)"""
import typing


@typing.runtime_checkable
class Sized(typing.Protocol):
    def __len__(self) -> int: ...


assert isinstance([1, 2, 3], Sized) is True, "a list has __len__, so it satisfies Sized"
assert isinstance(5, Sized) is False, "an int has no __len__, so it does not satisfy Sized"
print("runtime_checkable_protocol_isinstance OK")
