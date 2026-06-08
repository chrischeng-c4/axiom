# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "index_uses_dunder_index"
# subject = "operator.index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.index: index returns the integer from a type's __index__ and passes true ints through unchanged"""
import operator

class HasIndex:
    def __index__(self):
        return 7


assert operator.index(HasIndex()) == 7, "index via __index__"
assert operator.index(0) == 0, "index of int 0"
assert operator.index(42) == 42, "index of int"

print("index_uses_dunder_index OK")
