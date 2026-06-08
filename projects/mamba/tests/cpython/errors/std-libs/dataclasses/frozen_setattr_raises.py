# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "frozen_setattr_raises"
# subject = "dataclasses.FrozenInstanceError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.FrozenInstanceError: frozen_setattr_raises (errors)."""
import dataclasses

@dataclasses.dataclass(frozen=True)
class _P:
    x: int
    y: int

_p = _P(1, 2)

_raised = False
try:
    setattr(_p, 'x', 10)
except dataclasses.FrozenInstanceError:
    _raised = True
assert _raised, "frozen_setattr_raises: expected dataclasses.FrozenInstanceError"
print("frozen_setattr_raises OK")
