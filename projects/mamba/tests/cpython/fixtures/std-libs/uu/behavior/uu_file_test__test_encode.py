# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uu"
# dimension = "behavior"
# case = "uu_file_test__test_encode"
# subject = "cpython.test_uu.UUFileTest.test_encode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uu.py::UUFileTest::test_encode
"""Auto-ported test: UUFileTest::test_encode (CPython 3.12 oracle)."""


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
self_tmpin = os_helper.TESTFN_ASCII + 'i'
self_tmpout = os_helper.TESTFN_ASCII + 'o'
pass
pass
with open(self_tmpin, 'wb') as fin:
    fin.write(plaintext)
with open(self_tmpin, 'rb') as fin:
    with open(self_tmpout, 'wb') as fout:
        uu.encode(fin, fout, self_tmpin, mode=420)
with open(self_tmpout, 'rb') as fout:
    s = fout.read()

assert s == encodedtextwrapped(420, self_tmpin)
uu.encode(self_tmpin, self_tmpout, self_tmpin, mode=420)
with open(self_tmpout, 'rb') as fout:
    s = fout.read()

assert s == encodedtextwrapped(420, self_tmpin)
print("UUFileTest::test_encode: ok")
