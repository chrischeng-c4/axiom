# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "unique_decorator_rejects_aliases"
# subject = "enum.unique"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.unique: @enum.unique on a class containing a duplicate value (an alias) raises ValueError; an all-distinct class is accepted"""
import enum


# @unique rejects a class that contains an alias (duplicate value).
_raised = False
try:
    @enum.unique
    class Dirty(enum.Enum):
        ONE = 1
        TWO = 2
        ALSO_ONE = 1
except ValueError:
    _raised = True
assert _raised, "@unique on a class with aliases must raise ValueError"


# @unique accepts a class whose values are all distinct.
@enum.unique
class Clean(enum.Enum):
    A = 1
    B = 2
    C = 3


assert len(list(Clean)) == 3, "clean unique enum keeps all 3 members"

print("unique_decorator_rejects_aliases OK")
