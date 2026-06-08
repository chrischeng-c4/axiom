# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "binary_funcs_reject_bad_first_before_second"
# subject = "math.atan2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: atan2/copysign/remainder validate the first operand's type before touching the second; a bad first arg raises TypeError without ever calling __float__ on the second operand (bpo-39871 regression)"""
import math

class Tripwire:
    """Raises if __float__ is ever invoked, recording the attempt."""

    def __init__(self):
        self.converted = False

    def __float__(self):
        self.converted = True
        raise ZeroDivisionError("__float__ should not have been called")


for func in (math.atan2, math.copysign, math.remainder):
    probe = Tripwire()
    _raised = False
    try:
        func("not a number", probe)
    except TypeError:
        _raised = True
    assert _raised, f"{func.__name__}: bad first arg raises TypeError"
    assert not probe.converted, f"{func.__name__}: second arg left untouched"

print("binary_funcs_reject_bad_first_before_second OK")
