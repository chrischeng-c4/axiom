# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "protocol_direct_instantiation_raises"
# subject = "typing.Protocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Protocol: protocol_direct_instantiation_raises (errors)."""
import typing

class _DirectProto(typing.Protocol):
    def draw(self) -> None: ...


_raised = False
try:
    _DirectProto()
except TypeError:
    _raised = True
assert _raised, "protocol_direct_instantiation_raises: expected TypeError"
print("protocol_direct_instantiation_raises OK")
