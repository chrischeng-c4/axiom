# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isroutine_function_builtin_singledispatch"
# subject = "inspect.isroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isroutine: isroutine is True for a function, a builtin, and a functools.singledispatch wrapper, False for an int"""
import functools
import inspect

def f():
    pass

assert inspect.isroutine(f), "function is routine"
assert inspect.isroutine(len), "builtin is routine"
assert inspect.isroutine(functools.singledispatch(f)), "singledispatch is routine"
assert not inspect.isroutine(42), "int is not routine"

print("isroutine_function_builtin_singledispatch OK")
