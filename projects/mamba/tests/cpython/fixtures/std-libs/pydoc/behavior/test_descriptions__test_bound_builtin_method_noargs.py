# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "test_descriptions__test_bound_builtin_method_noargs"
# subject = "cpython.test_pydoc.TestDescriptions.test_bound_builtin_method_noargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pydoc.py::TestDescriptions::test_bound_builtin_method_noargs
"""Auto-ported test: TestDescriptions::test_bound_builtin_method_noargs (CPython 3.12 oracle)."""


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
import test.support
import types
import typing
import unittest
import unittest.mock
import urllib.parse
import xml.etree
import xml.etree.ElementTree
import textwrap
from io import StringIO
from collections import namedtuple
from urllib.request import urlopen, urlcleanup
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure, spawn_python
from test.support import threading_helper
from test.support import reap_children, captured_output, captured_stdout, captured_stderr, is_emscripten, is_wasi, requires_docstrings, MISSING_C_DOCSTRINGS
from test.support.os_helper import TESTFN, rmtree, unlink
from test.test_pydoc import pydoc_mod
from test.test_pydoc import pydocfodder


class nonascii:
    """Це не латиниця"""
    pass

if test.support.HAVE_DOCSTRINGS:
    expected_data_docstrings = ('dictionary for instance variables', 'list of weak references to the object') * 2
else:
    expected_data_docstrings = ('', '', '', '')

expected_text_pattern = "\nNAME\n    test.test_pydoc.pydoc_mod - This is a test module for test_pydoc\n%s\nCLASSES\n    builtins.object\n        A\n        B\n        C\n\n    class A(builtins.object)\n     |  Hello and goodbye\n     |\n     |  Methods defined here:\n     |\n     |  __init__()\n     |      Wow, I have no function!\n     |\n     |  ----------------------------------------------------------------------\n     |  Data descriptors defined here:\n     |\n     |  __dict__%s\n     |\n     |  __weakref__%s\n\n    class B(builtins.object)\n     |  Data descriptors defined here:\n     |\n     |  __dict__%s\n     |\n     |  __weakref__%s\n     |\n     |  ----------------------------------------------------------------------\n     |  Data and other attributes defined here:\n     |\n     |  NO_MEANING = 'eggs'\n     |\n     |  __annotations__ = {'NO_MEANING': <class 'str'>}\n\n    class C(builtins.object)\n     |  Methods defined here:\n     |\n     |  get_answer(self)\n     |      Return say_no()\n     |\n     |  is_it_true(self)\n     |      Return self.get_answer()\n     |\n     |  say_no(self)\n     |\n     |  ----------------------------------------------------------------------\n     |  Class methods defined here:\n     |\n     |  __class_getitem__(item)\n     |\n     |  ----------------------------------------------------------------------\n     |  Data descriptors defined here:\n     |\n     |  __dict__\n     |      dictionary for instance variables\n     |\n     |  __weakref__\n     |      list of weak references to the object\n\nFUNCTIONS\n    doc_func()\n        This function solves all of the world's problems:\n        hunger\n        lack of Python\n        war\n\n    nodoc_func()\n\nDATA\n    __xyz__ = 'X, Y and Z'\n    c_alias = test.test_pydoc.pydoc_mod.C[int]\n    list_alias1 = typing.List[int]\n    list_alias2 = list[int]\n    type_union1 = typing.Union[int, str]\n    type_union2 = int | str\n\nVERSION\n    1.2.3.4\n\nAUTHOR\n    Benjamin Peterson\n\nCREDITS\n    Nobody\n\nFILE\n    %s\n".strip()

expected_text_data_docstrings = tuple(('\n     |      ' + s if s else '' for s in expected_data_docstrings))

html2text_of_expected = "\ntest.test_pydoc.pydoc_mod (version 1.2.3.4)\nThis is a test module for test_pydoc\n\nModules\n    types\n    typing\n\nClasses\n    builtins.object\n    A\n    B\n    C\n\nclass A(builtins.object)\n    Hello and goodbye\n\n    Methods defined here:\n        __init__()\n            Wow, I have no function!\n    ----------------------------------------------------------------------\n    Data descriptors defined here:\n        __dict__\n            dictionary for instance variables\n        __weakref__\n            list of weak references to the object\n\nclass B(builtins.object)\n    Data descriptors defined here:\n        __dict__\n            dictionary for instance variables\n        __weakref__\n            list of weak references to the object\n    ----------------------------------------------------------------------\n    Data and other attributes defined here:\n        NO_MEANING = 'eggs'\n        __annotations__ = {'NO_MEANING': <class 'str'>}\n\n\nclass C(builtins.object)\n    Methods defined here:\n        get_answer(self)\n            Return say_no()\n        is_it_true(self)\n            Return self.get_answer()\n        say_no(self)\n    ----------------------------------------------------------------------\n    Class methods defined here:\n        __class_getitem__(item)\n    ----------------------------------------------------------------------\n    Data descriptors defined here:\n        __dict__\n            dictionary for instance variables\n        __weakref__\n             list of weak references to the object\n\nFunctions\n    doc_func()\n        This function solves all of the world's problems:\n        hunger\n        lack of Python\n        war\n    nodoc_func()\n\nData\n    __xyz__ = 'X, Y and Z'\n    c_alias = test.test_pydoc.pydoc_mod.C[int]\n    list_alias1 = typing.List[int]\n    list_alias2 = list[int]\n    type_union1 = typing.Union[int, str]\n    type_union2 = int | str\n\nAuthor\n    Benjamin Peterson\n\nCredits\n    Nobody\n"

expected_html_data_docstrings = tuple((s.replace(' ', '&nbsp;') for s in expected_data_docstrings))

missing_pattern = 'No Python documentation found for %r.\nUse help() to get the interactive help utility.\nUse help(str) for help on the str class.'.replace('\n', os.linesep)

badimport_pattern = 'problem in %s - ModuleNotFoundError: No module named %r'

expected_dynamicattribute_pattern = "\nHelp on class DA in module %s:\n\nclass DA(builtins.object)\n |  Data descriptors defined here:\n |\n |  __dict__%s\n |\n |  __weakref__%s\n |\n |  ham\n |\n |  ----------------------------------------------------------------------\n |  Data and other attributes inherited from Meta:\n |\n |  ham = 'spam'\n".strip()

expected_virtualattribute_pattern1 = '\nHelp on class Class in module %s:\n\nclass Class(builtins.object)\n |  Data and other attributes inherited from Meta:\n |\n |  LIFE = 42\n'.strip()

expected_virtualattribute_pattern2 = '\nHelp on class Class1 in module %s:\n\nclass Class1(builtins.object)\n |  Data and other attributes inherited from Meta1:\n |\n |  one = 1\n'.strip()

expected_virtualattribute_pattern3 = '\nHelp on class Class2 in module %s:\n\nclass Class2(Class1)\n |  Method resolution order:\n |      Class2\n |      Class1\n |      builtins.object\n |\n |  Data and other attributes inherited from Meta1:\n |\n |  one = 1\n |\n |  ----------------------------------------------------------------------\n |  Data and other attributes inherited from Meta3:\n |\n |  three = 3\n |\n |  ----------------------------------------------------------------------\n |  Data and other attributes inherited from Meta2:\n |\n |  two = 2\n'.strip()

expected_missingattribute_pattern = "\nHelp on class C in module %s:\n\nclass C(builtins.object)\n |  Data and other attributes defined here:\n |\n |  here = 'present!'\n".strip()

def run_pydoc(module_name, *args, **env):
    """
    Runs pydoc on the specified module. Returns the stripped
    output of pydoc.
    """
    args = args + (module_name,)
    rc, out, err = assert_python_ok('-B', pydoc.__file__, *args, **env)
    return out.strip()

def run_pydoc_fail(module_name, *args, **env):
    """
    Runs pydoc on the specified module expecting a failure.
    """
    args = args + (module_name,)
    rc, out, err = assert_python_failure('-B', pydoc.__file__, *args, **env)
    return out.strip()

def get_pydoc_html(module):
    """Returns pydoc generated output as html"""
    doc = pydoc.HTMLDoc()
    output = doc.docmodule(module)
    loc = doc.getdocloc(pydoc_mod) or ''
    if loc:
        loc = '<br><a href="' + loc + '">Module Docs</a>'
    return (output.strip(), loc)

def clean_text(doc):
    return re.sub('\x08.', '', doc)

def get_pydoc_link(module):
    """Returns a documentation web link of a module"""
    abspath = os.path.abspath
    dirname = os.path.dirname
    basedir = dirname(dirname(dirname(abspath(__file__))))
    doc = pydoc.TextDoc()
    loc = doc.getdocloc(module, basedir=basedir)
    return loc

def get_pydoc_text(module):
    """Returns pydoc generated output as text"""
    doc = pydoc.TextDoc()
    loc = doc.getdocloc(pydoc_mod) or ''
    if loc:
        loc = '\nMODULE DOCS\n    ' + loc + '\n'
    output = doc.docmodule(module)
    output = clean_text(output)
    return (output.strip(), loc)

def get_html_title(text):
    header, _, _ = text.partition('</head>')
    _, _, title = header.partition('<title>')
    title, _, _ = title.partition('</title>')
    return title

def html2text(html):
    """A quick and dirty implementation of html2text.

    Tailored for pydoc tests only.
    """
    html = html.replace('<dd>', '\n')
    html = html.replace('<hr>', '-' * 70)
    html = re.sub('<.*?>', '', html)
    html = pydoc.replace(html, '&nbsp;', ' ', '&gt;', '>', '&lt;', '<')
    return html

def setUpModule():
    thread_info = threading_helper.threading_setup()
    unittest.addModuleCleanup(threading_helper.threading_cleanup, *thread_info)
    unittest.addModuleCleanup(reap_children)


# --- test body ---
def _get_summary_line(o):
    text = pydoc.plain(pydoc.render_doc(o))
    lines = text.split('\n')
    assert len(lines) >= 2
    return lines[2]

def _get_summary_lines(o):
    text = pydoc.plain(pydoc.render_doc(o))
    lines = text.split('\n')
    return '\n'.join(lines[2:])

assert _get_summary_line(''.lower) == 'lower() method of builtins.str instance'
print("TestDescriptions::test_bound_builtin_method_noargs: ok")
