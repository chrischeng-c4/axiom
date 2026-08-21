# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "ismethod_bound_vs_unbound"
# subject = "inspect.ismethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.ismethod: ismethod is True for a bound method on an instance, False for the plain function accessed on the class"""
import inspect

class _Owner:
    def method(self):
        pass

_o = _Owner()
assert inspect.ismethod(_o.method), "ismethod(bound method)"
assert not inspect.ismethod(_Owner.method), "not ismethod(unbound)"

print("ismethod_bound_vs_unbound OK")
