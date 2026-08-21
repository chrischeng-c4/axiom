# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamicclassattribute"
# dimension = "behavior"
# case = "property_tests__test_abstract_virtual"
# subject = "cpython.test_dynamicclassattribute.PropertyTests.test_abstract_virtual"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dynamicclassattribute.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dynamicclassattribute.py::PropertyTests::test_abstract_virtual
"""Auto-ported test: PropertyTests::test_abstract_virtual (CPython 3.12 oracle)."""


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

try:
    ClassWithAbstractVirtualProperty()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    ClassWithPropertyAbstractVirtual()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class APV(ClassWithPropertyAbstractVirtual):
    pass

try:
    APV()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class AVP(ClassWithAbstractVirtualProperty):
    pass

try:
    AVP()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class Okay1(ClassWithAbstractVirtualProperty):

    @DynamicClassAttribute
    def color(self):
        return self._color

    def __init__(self):
        self._color = 'cyan'
try:
    Okay1.color
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert Okay1().color == 'cyan'

class Okay2(ClassWithAbstractVirtualProperty):

    @DynamicClassAttribute
    def color(self):
        return self._color

    def __init__(self):
        self._color = 'magenta'
try:
    Okay2.color
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert Okay2().color == 'magenta'
print("PropertyTests::test_abstract_virtual: ok")
