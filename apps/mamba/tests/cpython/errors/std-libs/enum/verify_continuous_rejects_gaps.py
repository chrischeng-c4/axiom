# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "verify_continuous_rejects_gaps"
# subject = "enum.verify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.verify: @verify(CONTINUOUS) on an Enum whose integer values leave a gap (3, 4, 11) raises ValueError"""
import enum


# @verify(CONTINUOUS) requires the integer values to form a gap-free range.
_raised = False
try:
    @enum.verify(enum.CONTINUOUS)
    class Gappy(enum.Enum):
        FIRST = 3
        SECOND = 4
        THIRD = 11  # gap between 4 and 11
except ValueError:
    _raised = True
assert _raised, "@verify(CONTINUOUS) on a gappy range must raise ValueError"

print("verify_continuous_rejects_gaps OK")
