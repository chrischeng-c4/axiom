# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "overload_only_stub_call_raises"
# subject = "typing.overload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.overload: overload_only_stub_call_raises (errors)."""
import typing

@typing.overload
def _only_stub(x: int) -> int: ...
@typing.overload
def _only_stub(x: str) -> str: ...


_raised = False
try:
    _only_stub(1)
except NotImplementedError:
    _raised = True
assert _raised, "overload_only_stub_call_raises: expected NotImplementedError"
print("overload_only_stub_call_raises OK")
