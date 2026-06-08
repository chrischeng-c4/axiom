# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "call_invokes_callable"
# subject = "operator.call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.call: call(obj, *args, **kwargs) invokes the callable forwarding positional and keyword arguments exactly"""
import operator

def collect(*args, **kwargs):
    return (args, kwargs)


assert operator.call(collect) == ((), {}), "call no args"
assert operator.call(collect, 0, 1) == ((0, 1), {}), "call positional"
assert operator.call(collect, a=2, b=3) == ((), {"a": 2, "b": 3}), "call kwargs"
assert operator.call(collect, 0, a=2) == ((0,), {"a": 2}), "call mixed"

print("call_invokes_callable OK")
