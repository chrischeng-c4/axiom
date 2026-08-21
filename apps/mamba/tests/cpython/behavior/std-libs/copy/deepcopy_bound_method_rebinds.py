# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "deepcopy_bound_method_rebinds"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: deepcopying an object that stores one of its own bound methods rebinds the method's __self__ to the copy, not the original"""
import copy


class Bound:
    def m(self):
        return self


b = Bound()
b.b = b.m  # store a bound method of self as an attribute
bd = copy.deepcopy(b)
assert bd.b.__self__ is bd, "deepcopy rebinds the bound method __self__ to the copy"
assert bd.m == bd.b, "the rebound method equals a freshly-bound method on the copy"

print("deepcopy_bound_method_rebinds OK")
