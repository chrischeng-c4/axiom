# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "methodcaller_propagates_method_argerror"
# subject = "operator.methodcaller"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: the bound method's own argument error propagates unchanged: methodcaller('add_first_two')(t) with too few args raises IndexError from the method body"""
import operator


class Target:
    def add_first_two(self, *args, **kwds):
        return args[0] + args[1]


_raised = False
try:
    operator.methodcaller("add_first_two")(Target())
except IndexError:
    _raised = True
assert _raised, "expected IndexError from the method body"
print("methodcaller_propagates_method_argerror OK")
