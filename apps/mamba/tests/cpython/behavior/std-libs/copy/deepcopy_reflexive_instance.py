# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_reflexive_instance"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopy of an instance that stores a reference to itself rebuilds the self-reference through the memo"""
import copy


class Cyclic:
    pass


c = Cyclic()
c.foo = c  # self-referential instance attribute
cd = copy.deepcopy(c)
assert cd is not c, "deepcopy is a new instance"
assert cd.foo is cd, "the self-reference is rebuilt to point at the copy, not the original"

print("deepcopy_reflexive_instance OK")
