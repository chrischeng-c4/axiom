# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "errors"
# case = "nonruntime_protocol_isinstance_raises"
# subject = "typing.Protocol"
# kind = "mechanical"
# xfail = "mamba does not raise on isinstance against a non-runtime-checkable Protocol (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Protocol: nonruntime_protocol_isinstance_raises (errors)."""
import typing

class _NonRuntimeProto(typing.Protocol):
    def foo(self) -> int: ...


_raised = False
try:
    isinstance(1, _NonRuntimeProto)
except TypeError:
    _raised = True
assert _raised, "nonruntime_protocol_isinstance_raises: expected TypeError"
print("nonruntime_protocol_isinstance_raises OK")
