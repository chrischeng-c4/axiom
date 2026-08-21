# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "non_runtime_protocol_isinstance_raises"
# subject = "typing.Protocol"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.Protocol: isinstance against a Protocol that is NOT @runtime_checkable raises TypeError (instance checks need the @runtime_checkable decorator)"""
import typing


class P(typing.Protocol):
    def method(self) -> int: ...


_raised = False
try:
    isinstance(1, P)
except TypeError:
    _raised = True
assert _raised, "non_runtime_protocol_isinstance_raises: expected TypeError"
print("non_runtime_protocol_isinstance_raises OK")
