# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isdatadescriptor_property_and_slot"
# subject = "inspect.isdatadescriptor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isdatadescriptor: isdatadescriptor is True for a property and a __slots__ member descriptor, False for a class and a plain function"""
import inspect

class WithProp:
    @property
    def a_property(self):
        return 1

class Slotted:
    __slots__ = ("x",)

assert inspect.isdatadescriptor(WithProp.a_property), "property is data descriptor"
assert inspect.isdatadescriptor(Slotted.x), "slot is data descriptor"
assert not inspect.isdatadescriptor(WithProp), "class is not data descriptor"
assert not inspect.isdatadescriptor(lambda: 0), "function is not data descriptor"

print("isdatadescriptor_property_and_slot OK")
