# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getattr_static_does_not_fire_descriptors"
# subject = "inspect.getattr_static"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getattr_static: getattr_static() reads an attribute without firing descriptors: a property returns the property object, a slot returns the member descriptor, a default applies when missing"""
import inspect

class _Desc:
    cls_attr = object()

    @property
    def prop(self):
        return 42

_d = _Desc()
# Property: static access returns the descriptor itself, not 42.
_static = inspect.getattr_static(_d, "prop")
assert isinstance(_static, property), f"static prop = {type(_static)!r}"
# Plain class attribute: same identity.
assert inspect.getattr_static(_d, "cls_attr") is _Desc.cls_attr, "static class attr"
# Missing attribute: default applies, otherwise AttributeError.
assert inspect.getattr_static(_d, "missing", "fallback") == "fallback", "static default"
_raised = False
try:
    inspect.getattr_static(_d, "missing")
except AttributeError:
    _raised = True
assert _raised, "expected AttributeError without default"

# Slot: dynamic read returns the value, static read returns the member descriptor.
class Slotted:
    __slots__ = ("x",)

s = Slotted()
s.x = 7
assert getattr(s, "x") == 7, "dynamic slot value"
assert inspect.isdatadescriptor(inspect.getattr_static(s, "x")), "static slot is descriptor"

print("getattr_static_does_not_fire_descriptors OK")
