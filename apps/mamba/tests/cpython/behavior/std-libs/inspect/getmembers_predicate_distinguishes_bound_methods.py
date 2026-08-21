# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getmembers_predicate_distinguishes_bound_methods"
# subject = "inspect.getmembers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getmembers: getmembers(predicate=ismethod) yields bound methods on an instance but not the plain function on the class"""
import inspect

class _Holder:
    def m(self):
        pass

# On the class, m is a plain function (not a bound method).
assert ("m", _Holder.m) in inspect.getmembers(_Holder), "function in class members"
assert ("m", _Holder.m) not in inspect.getmembers(_Holder, inspect.ismethod), (
    "class function is not ismethod"
)
# On an instance, m is a bound method.
_h = _Holder()
assert ("m", _h.m) in inspect.getmembers(_h, inspect.ismethod), "instance method ismethod"

print("getmembers_predicate_distinguishes_bound_methods OK")
