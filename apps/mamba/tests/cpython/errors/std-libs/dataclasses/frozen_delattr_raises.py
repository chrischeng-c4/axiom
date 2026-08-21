# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "frozen_delattr_raises"
# subject = "dataclasses.FrozenInstanceError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.FrozenInstanceError: frozen_delattr_raises (errors)."""
import dataclasses

@dataclasses.dataclass(frozen=True)
class _P:
    x: int
    y: int

_p = _P(1, 2)

_raised = False
try:
    delattr(_p, 'x')
except dataclasses.FrozenInstanceError:
    _raised = True
assert _raised, "frozen_delattr_raises: expected dataclasses.FrozenInstanceError"
print("frozen_delattr_raises OK")
