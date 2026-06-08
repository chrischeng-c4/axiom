# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uu"
# dimension = "behavior"
# case = "uu_std_io_test__test_encode"
# subject = "cpython.test_uu.UUStdIOTest.test_encode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uu.py::UUStdIOTest::test_encode
"""Auto-ported test: UUStdIOTest::test_encode (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper, warnings_helper
import os
import stat
import sys
import io


'\nTests for uu module.\nNick Mathewson\n'

uu = warnings_helper.import_deprecated('uu')

plaintext = b'The symbols on top of your keyboard are !@#$%^&*()_+|~\n'

encodedtext = b'M5&AE(\'-Y;6)O;\',@;VX@=&]P(&]F(\'EO=7(@:V5Y8F]A<F0@87)E("% (R0E\n*7B8J*"E?*WQ^"@  '

class FakeIO(io.TextIOWrapper):
    """Text I/O implementation using an in-memory buffer.

    Can be a used as a drop-in replacement for sys.stdin and sys.stdout.
    """

    def __init__(self, initial_value='', encoding='utf-8', errors='strict', newline='\n'):
        super(FakeIO, self).__init__(io.BytesIO(), encoding=encoding, errors=errors, newline=newline)
        self._encoding = encoding
        self._errors = errors
        if initial_value:
            if not isinstance(initial_value, str):
                initial_value = str(initial_value)
            self.write(initial_value)
            self.seek(0)

    def getvalue(self):
        self.flush()
        return self.buffer.getvalue().decode(self._encoding, self._errors)

def encodedtextwrapped(mode, filename, backtick=False):
    if backtick:
        res = bytes('begin %03o %s\n' % (mode, filename), 'ascii') + encodedtext.replace(b' ', b'`') + b'\n`\nend\n'
    else:
        res = bytes('begin %03o %s\n' % (mode, filename), 'ascii') + encodedtext + b'\n \nend\n'
    return res


# --- test body ---
self_stdin = sys.stdin
self_stdout = sys.stdout
sys.stdin = FakeIO(plaintext.decode('ascii'))
sys.stdout = FakeIO()
uu.encode('-', '-', 't1', 438)

assert sys.stdout.getvalue() == encodedtextwrapped(438, 't1').decode('ascii')
print("UUStdIOTest::test_encode: ok")
