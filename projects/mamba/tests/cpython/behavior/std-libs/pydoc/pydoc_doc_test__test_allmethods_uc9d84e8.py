# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_doc_test__test_allmethods_uc9d84e8"
# subject = "cpython.test_pydoc.PydocDocTest.test_allmethods"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import os
import sys
import contextlib
import importlib.util
import inspect
import io
import pydoc
import py_compile
import keyword
import _pickle
import pkgutil
import re
import stat
import tempfile
import types
import typing
import urllib.parse
import xml.etree
import xml.etree.ElementTree
import textwrap
from io import StringIO
from collections import namedtuple
from urllib.request import urlopen, urlcleanup
maxDiff = None

class TestClass(object):

    def method_returning_true(self):
        return True
expected = dict(vars(object))
expected['method_returning_true'] = TestClass.method_returning_true
del expected['__doc__']
del expected['__class__']
expected['__subclasshook__'] = TestClass.__subclasshook__
expected['__init_subclass__'] = TestClass.__init_subclass__
methods = pydoc.allmethods(TestClass)
assert methods == expected

print("PydocDocTest::test_allmethods: ok")
