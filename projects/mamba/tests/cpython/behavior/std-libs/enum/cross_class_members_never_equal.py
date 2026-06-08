# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "cross_class_members_never_equal"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: members of two different Enum classes are never equal even with identical values (Season.SPRING != Part.SPRING)"""
import enum

class Season(enum.Enum):
    SPRING = 1
    SUMMER = 2

class Part(enum.Enum):
    SPRING = 1
    SUMMER = 2

assert Season.SPRING != Part.SPRING
assert Season.SPRING.value == Part.SPRING.value == 1
assert Season.SPRING == Season.SPRING
assert Season.SPRING != 1   # a plain Enum member is not its raw value

print("cross_class_members_never_equal OK")
