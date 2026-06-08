# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bind_captures_args_in_order"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: Signature.bind() captures args in declaration order, folding extras into *args/**kwargs, and BoundArguments.args/.kwargs split them"""
import inspect

def mixed(a, *args, b, z=100, **kwargs):
    pass

sig = inspect.signature(mixed)
ba = sig.bind(10, 20, b=30, c=40)
assert tuple(ba.arguments.items()) == (
    ("a", 10),
    ("args", (20,)),
    ("b", 30),
    ("kwargs", {"c": 40}),
), f"arguments = {tuple(ba.arguments.items())!r}"
assert ba.args == (10, 20), f"ba.args = {ba.args!r}"
assert ba.kwargs == {"b": 30, "c": 40}, f"ba.kwargs = {ba.kwargs!r}"

print("bind_captures_args_in_order OK")
