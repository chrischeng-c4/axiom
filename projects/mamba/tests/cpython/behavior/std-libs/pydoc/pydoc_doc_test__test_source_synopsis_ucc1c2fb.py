# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_doc_test__test_source_synopsis_ucc1c2fb"
# subject = "cpython.test_pydoc.PydocDocTest.test_source_synopsis"
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

def check(source, expected, encoding=None):
    if isinstance(source, str):
        source_file = StringIO(source)
    else:
        source_file = io.TextIOWrapper(io.BytesIO(source), encoding=encoding)
    with source_file:
        result = pydoc.source_synopsis(source_file)
        assert result == expected
check('"""Single line docstring."""', 'Single line docstring.')
check('"""First line of docstring.\nSecond line.\nThird line."""', 'First line of docstring.')
check('"""First line of docstring.\\nSecond line.\\nThird line."""', 'First line of docstring.')
check('"""  Whitespace around docstring.  """', 'Whitespace around docstring.')
check('import sys\n"""No docstring"""', None)
check('  \n"""Docstring after empty line."""', 'Docstring after empty line.')
check('# Comment\n"""Docstring after comment."""', 'Docstring after comment.')
check('  # Indented comment\n"""Docstring after comment."""', 'Docstring after comment.')
check('""""""', '')
check('', None)
check('"""Embedded\x00null byte"""', None)
check('"""Embedded null byte"""\x00', None)
check('"""Café and résumé."""', 'Café and résumé.')
check("'''Triple single quotes'''", 'Triple single quotes')
check('"Single double quotes"', 'Single double quotes')
check("'Single single quotes'", 'Single single quotes')
check('"""split\\\nline"""', 'splitline')
check('"""Unrecognized escape \\sequence"""', 'Unrecognized escape \\sequence')
check('"""Invalid escape seq\\uence"""', None)
check('r"""Raw \\stri\\ng"""', 'Raw \\stri\\ng')
check('b"""Bytes literal"""', None)
check('f"""f-string"""', None)
check('"""Concatenated""" \\\n"string" \'literals\'', 'Concatenatedstringliterals')
check('"""String""" + """expression"""', None)
check('("""In parentheses""")', 'In parentheses')
check('("""Multiple lines """\n"""in parentheses""")', 'Multiple lines in parentheses')
check('()', None)
check(b'# coding: iso-8859-15\n"""\xa4uro sign"""', '€uro sign', encoding='iso-8859-15')
check(b'"""\xa4"""', None, encoding='utf-8')
with tempfile.NamedTemporaryFile(mode='w+', encoding='utf-8') as temp_file:
    temp_file.write('"""Real file test."""\n')
    temp_file.flush()
    temp_file.seek(0)
    result = pydoc.source_synopsis(temp_file)
    assert result == 'Real file test.'

print("PydocDocTest::test_source_synopsis: ok")
