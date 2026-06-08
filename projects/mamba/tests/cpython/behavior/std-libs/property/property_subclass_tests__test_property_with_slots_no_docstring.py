# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "property"
# dimension = "behavior"
# case = "property_subclass_tests__test_property_with_slots_no_docstring"
# subject = "cpython.test_property.PropertySubclassTests.test_property_with_slots_no_docstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_property.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_property.py::PropertySubclassTests::test_property_with_slots_no_docstring
"""Auto-ported test: PropertySubclassTests::test_property_with_slots_no_docstring (CPython 3.12 oracle)."""


import sys
import unittest
from test import support


class PropertyBase(Exception):
    pass

class PropertyGet(PropertyBase):
    pass

class PropertySet(PropertyBase):
    pass

class PropertyDel(PropertyBase):
    pass

class BaseClass(object):

    def __init__(self):
        self._spam = 5

    @property
    def spam(self):
        """BaseClass.getter"""
        return self._spam

    @spam.setter
    def spam(self, value):
        self._spam = value

    @spam.deleter
    def spam(self):
        del self._spam

class SubClass(BaseClass):

    @BaseClass.spam.getter
    def spam(self):
        """SubClass.getter"""
        raise PropertyGet(self._spam)

    @spam.setter
    def spam(self, value):
        raise PropertySet(self._spam)

    @spam.deleter
    def spam(self):
        raise PropertyDel(self._spam)

class PropertyDocBase(object):
    _spam = 1

    def _get_spam(self):
        return self._spam
    spam = property(_get_spam, doc='spam spam spam')

class PropertyDocSub(PropertyDocBase):

    @PropertyDocBase.spam.getter
    def spam(self):
        """The decorator does not use this doc string"""
        return self._spam

class PropertySubNewGetter(BaseClass):

    @BaseClass.spam.getter
    def spam(self):
        """new docstring"""
        return 5

class PropertyNewGetter(object):

    @property
    def spam(self):
        """original docstring"""
        return 1

    @spam.getter
    def spam(self):
        """new docstring"""
        return 8

class PropertySub(property):
    """This is a subclass of property"""

class PropertySubWoDoc(property):
    pass

class PropertySubSlots(property):
    """This is a subclass of property that defines __slots__"""
    __slots__ = ()


# --- test body ---
class slotted_prop(property):
    __slots__ = ('foo',)
p = slotted_prop()

assert getattr(p, '__doc__', None) is None

def undocumented_getter():
    return 4
p = slotted_prop(undocumented_getter)

assert getattr(p, '__doc__', None) is None
print("PropertySubclassTests::test_property_with_slots_no_docstring: ok")
