# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bound_arguments_apply_defaults"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: BoundArguments.apply_defaults() fills missing defaults including empty *args and **kwargs"""
import inspect

def defs(a, b=1, *args, c=7, **kw):
    pass

sig2 = inspect.signature(defs)
ba2 = sig2.bind(20)
ba2.apply_defaults()
assert list(ba2.arguments.items()) == [
    ("a", 20),
    ("b", 1),
    ("args", ()),
    ("c", 7),
    ("kw", {}),
], f"defaults = {list(ba2.arguments.items())!r}"

print("bound_arguments_apply_defaults OK")
