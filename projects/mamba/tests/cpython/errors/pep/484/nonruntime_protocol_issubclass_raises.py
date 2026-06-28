# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "nonruntime_protocol_issubclass_raises"
# subject = "typing.Protocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Protocol: issubclass against a non-runtime Protocol raises TypeError."""
from typing import Protocol


class _NonRuntimeProto(Protocol):
    def close(self) -> None: ...


class File:
    def close(self):
        pass


_raised = False
try:
    issubclass(File, _NonRuntimeProto)
except TypeError:
    _raised = True
assert _raised, "nonruntime_protocol_issubclass_raises: expected TypeError"

print("nonruntime_protocol_issubclass_raises OK")
