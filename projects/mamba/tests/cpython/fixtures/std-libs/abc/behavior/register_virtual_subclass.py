# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "register_virtual_subclass"
# subject = "abc.ABCMeta.register"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta.register: register() makes an unrelated class a virtual subclass for issubclass and isinstance without inheritance"""
import abc


class Interface(abc.ABC):
    pass


class Impl:
    pass  # no inheritance relationship to Interface


# Before registration, Impl is unrelated.
assert not issubclass(Impl, Interface), "Impl is not a subclass before register"

Interface.register(Impl)
assert issubclass(Impl, Interface), "registered class is a virtual subclass"
assert isinstance(Impl(), Interface), "registered instance passes isinstance"

print("register_virtual_subclass OK")
