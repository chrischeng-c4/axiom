# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_wraps_bound_and_unbound_methods"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial wraps both an unbound (str.join) and a bound (''.join) method as the underlying callable"""
import functools

data = [str(i) for i in range(10)]

# Unbound method: the separator is the bound first positional.
join_unbound = functools.partial(str.join, "")
assert join_unbound(data) == "0123456789", "unbound str.join"

# Bound method: the partial wraps an already-bound callable.
join_bound = functools.partial("".join)
assert join_bound(data) == "0123456789", "bound ''.join"

print("partial_wraps_bound_and_unbound_methods OK")
