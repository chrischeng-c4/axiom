# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_encoding"
# subject = "cpython.test_compile.TestSpecifics.test_encoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_encoding
"""Auto-ported test: TestSpecifics::test_encoding (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
code = b'# -*- coding: badencoding -*-\npass\n'

try:
    compile(code, 'tmp', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
code = '# -*- coding: badencoding -*-\n"Â¤"\n'
compile(code, 'tmp', 'exec')

assert eval(code) == 'Â¤'
code = '"Â¤"\n'

assert eval(code) == 'Â¤'
code = b'"\xc2\xa4"\n'

assert eval(code) == '¤'
code = b'# -*- coding: latin1 -*-\n"\xc2\xa4"\n'

assert eval(code) == 'Â¤'
code = b'# -*- coding: utf-8 -*-\n"\xc2\xa4"\n'

assert eval(code) == '¤'
code = b'# -*- coding: iso8859-15 -*-\n"\xc2\xa4"\n'

assert eval(code) == 'Â€'
code = '"""\\\n# -*- coding: iso8859-15 -*-\nÂ¤"""\n'

assert eval(code) == '# -*- coding: iso8859-15 -*-\nÂ¤'
code = b'"""\\\n# -*- coding: iso8859-15 -*-\n\xc2\xa4"""\n'

assert eval(code) == '# -*- coding: iso8859-15 -*-\n¤'
print("TestSpecifics::test_encoding: ok")
