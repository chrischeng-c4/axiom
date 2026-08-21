# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uu"
# dimension = "behavior"
# case = "uu_test__test_no_directory_traversal"
# subject = "cpython.test_uu.UUTest.test_no_directory_traversal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_uu.py::UUTest::test_no_directory_traversal
"""Auto-ported test: UUTest::test_no_directory_traversal (CPython 3.12 oracle)."""


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
relative_bad = b'begin 644 ../../../../../../../../tmp/test1\n$86)C"@``\n`\nend\n'
try:
    uu.decode(io.BytesIO(relative_bad))
    raise AssertionError('expected uu.Error')
except uu.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('directory', str(_aR_e))
if os.altsep:
    relative_bad_bs = relative_bad.replace(b'/', b'\\')
    try:
        uu.decode(io.BytesIO(relative_bad_bs))
        raise AssertionError('expected uu.Error')
    except uu.Error as _aR_e:
        import re as _re_aR
        assert _re_aR.search('directory', str(_aR_e))
absolute_bad = b'begin 644 /tmp/test2\n$86)C"@``\n`\nend\n'
try:
    uu.decode(io.BytesIO(absolute_bad))
    raise AssertionError('expected uu.Error')
except uu.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('directory', str(_aR_e))
if os.altsep:
    absolute_bad_bs = absolute_bad.replace(b'/', b'\\')
    try:
        uu.decode(io.BytesIO(absolute_bad_bs))
        raise AssertionError('expected uu.Error')
    except uu.Error as _aR_e:
        import re as _re_aR
        assert _re_aR.search('directory', str(_aR_e))
print("UUTest::test_no_directory_traversal: ok")
