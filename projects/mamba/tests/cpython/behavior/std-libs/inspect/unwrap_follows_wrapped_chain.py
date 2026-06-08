# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "unwrap_follows_wrapped_chain"
# subject = "inspect.unwrap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.unwrap: unwrap() follows the __wrapped__ chain (one or many links) to the bottom function and honors a stop predicate"""
import functools
import inspect

def base(a, b):
    return a + b

# unwrap() follows the __wrapped__ chain (one or many links) to the bottom.
assert inspect.unwrap(functools.wraps(base)(lambda: None)) is base, "unwrap one"
chain = base
for _i in range(5):
    chain = functools.wraps(chain)(lambda: None)
assert inspect.unwrap(chain) is base, "unwrap several"

# unwrap() honors a stop predicate.
@functools.wraps(base)
def middle():
    pass

@functools.wraps(middle)
def outer():
    pass

# Mark middle only after wrapping so the flag is not copied onto outer.
middle.stop_here = 1
stopped = inspect.unwrap(outer, stop=lambda fn: hasattr(fn, "stop_here"))
assert stopped is middle, "unwrap stop predicate"

print("unwrap_follows_wrapped_chain OK")
