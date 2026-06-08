# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "property"
# dimension = "behavior"
# case = "property_tests__test_property_decorator_doc"
# subject = "cpython.test_property.PropertyTests.test_property_decorator_doc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_property.py::PropertyTests::test_property_decorator_doc
"""Auto-ported test: PropertyTests::test_property_decorator_doc (CPython 3.12 oracle)."""


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
base = PropertyDocBase()
sub = PropertyDocSub()

assert base.__class__.spam.__doc__ == 'spam spam spam'

assert sub.__class__.spam.__doc__ == 'spam spam spam'
print("PropertyTests::test_property_decorator_doc: ok")
