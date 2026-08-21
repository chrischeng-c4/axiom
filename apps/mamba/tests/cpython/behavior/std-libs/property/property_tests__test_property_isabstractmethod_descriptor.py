# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "property"
# dimension = "behavior"
# case = "property_tests__test_property_isabstractmethod_descriptor"
# subject = "cpython.test_property.PropertyTests.test_property___isabstractmethod__descriptor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_property.py::PropertyTests::test_property___isabstractmethod__descriptor
"""Auto-ported test: PropertyTests::test_property___isabstractmethod__descriptor (CPython 3.12 oracle)."""


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
for val in (True, False, [], [1], '', '1'):

    class C(object):

        def foo(self):
            pass
        foo.__isabstractmethod__ = val
        foo = property(foo)

    assert C.foo.__isabstractmethod__ is bool(val)

class NotBool(object):

    def __bool__(self):
        raise ValueError()
    __len__ = __bool__
try:

    class C(object):

        def foo(self):
            pass
        foo.__isabstractmethod__ = NotBool()
        foo = property(foo)
    C.foo.__isabstractmethod__
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PropertyTests::test_property___isabstractmethod__descriptor: ok")
