# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "ismodule_true_for_module_false_for_func"
# subject = "inspect.ismodule"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.ismodule: ismodule is True for an imported module, False for a function"""
import inspect
import math

def _myfunc(x):
    return x

assert inspect.ismodule(math), "ismodule(math)"
assert not inspect.ismodule(_myfunc), "not ismodule(func)"

print("ismodule_true_for_module_false_for_func OK")
