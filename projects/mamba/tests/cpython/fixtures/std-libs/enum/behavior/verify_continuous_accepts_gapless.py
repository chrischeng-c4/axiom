# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "verify_continuous_accepts_gapless"
# subject = "enum.verify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.verify: @verify(CONTINUOUS) accepts an Enum whose auto() values form a gap-free 1, 2, 3 range, and @unique accepts an all-distinct class"""
import enum

@enum.verify(enum.CONTINUOUS)
class Shade(enum.Enum):
    RED = enum.auto()
    GREEN = enum.auto()
    BLUE = enum.auto()

@enum.unique
class Clean(enum.Enum):
    A = 1
    B = 2
    C = 3

assert [m.value for m in Shade] == [1, 2, 3]
assert len(list(Clean)) == 3

print("verify_continuous_accepts_gapless OK")
