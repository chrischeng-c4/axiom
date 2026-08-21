# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isfunction_true_for_def_false_for_int"
# subject = "inspect.isfunction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isfunction: isfunction is True for a def, False for a non-function (int) and for a class"""
import inspect

def _myfunc(x):
    return x

class _MyClass:
    pass

assert inspect.isfunction(_myfunc), "isfunction(func)"
assert not inspect.isfunction(42), "not isfunction(int)"
assert not inspect.isfunction(_MyClass), "class is not function"

print("isfunction_true_for_def_false_for_int OK")
