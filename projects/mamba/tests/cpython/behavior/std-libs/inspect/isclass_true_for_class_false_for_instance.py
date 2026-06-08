# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isclass_true_for_class_false_for_instance"
# subject = "inspect.isclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isclass: isclass is True for a class object, False for an instance of it and for a function"""
import inspect

class _MyClass:
    pass

def _f():
    pass

assert inspect.isclass(_MyClass), "isclass(class)"
assert not inspect.isclass(_MyClass()), "not isclass(instance)"
assert not inspect.isclass(_f), "function is not class"

print("isclass_true_for_class_false_for_instance OK")
