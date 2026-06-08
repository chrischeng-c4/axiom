# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamicclassattribute"
# dimension = "behavior"
# case = "property_tests__test_property_isabstractmethod_descriptor"
# subject = "cpython.test_dynamicclassattribute.PropertyTests.test_property___isabstractmethod__descriptor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dynamicclassattribute.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dynamicclassattribute.py::PropertyTests::test_property___isabstractmethod__descriptor
"""Auto-ported test: PropertyTests::test_property___isabstractmethod__descriptor (CPython 3.12 oracle)."""


import abc
import sys
import unittest
from types import DynamicClassAttribute


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

    @DynamicClassAttribute
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
    spam = BaseClass.__dict__['spam']

    @spam.getter
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
    spam = DynamicClassAttribute(_get_spam, doc='spam spam spam')

class PropertyDocSub(PropertyDocBase):
    spam = PropertyDocBase.__dict__['spam']

    @spam.getter
    def spam(self):
        """The decorator does not use this doc string"""
        return self._spam

class PropertySubNewGetter(BaseClass):
    spam = BaseClass.__dict__['spam']

    @spam.getter
    def spam(self):
        """new docstring"""
        return 5

class PropertyNewGetter(object):

    @DynamicClassAttribute
    def spam(self):
        """original docstring"""
        return 1

    @spam.getter
    def spam(self):
        """new docstring"""
        return 8

class ClassWithAbstractVirtualProperty(metaclass=abc.ABCMeta):

    @DynamicClassAttribute
    @abc.abstractmethod
    def color():
        pass

class ClassWithPropertyAbstractVirtual(metaclass=abc.ABCMeta):

    @abc.abstractmethod
    @DynamicClassAttribute
    def color():
        pass

class PropertySub(DynamicClassAttribute):
    """This is a subclass of DynamicClassAttribute"""

class PropertySubSlots(DynamicClassAttribute):
    """This is a subclass of DynamicClassAttribute that defines __slots__"""
    __slots__ = ()


# --- test body ---
for val in (True, False, [], [1], '', '1'):

    class C(object):

        def foo(self):
            pass
        foo.__isabstractmethod__ = val
        foo = DynamicClassAttribute(foo)

    assert C.__dict__['foo'].__isabstractmethod__ is bool(val)

class NotBool(object):

    def __bool__(self):
        raise ValueError()
    __len__ = __bool__
try:

    class C(object):

        def foo(self):
            pass
        foo.__isabstractmethod__ = NotBool()
        foo = DynamicClassAttribute(foo)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("PropertyTests::test_property___isabstractmethod__descriptor: ok")
