# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "charmapcodec"
# dimension = "behavior"
# case = "charmap_codec_test__test_encodex"
# subject = "cpython.test_charmapcodec.CharmapCodecTest.test_encodex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_charmapcodec.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_charmapcodec.py::CharmapCodecTest::test_encodex
"""Auto-ported test: CharmapCodecTest::test_encodex (CPython 3.12 oracle)."""


import unittest
import codecs


' Python character mapping codec test\n\nThis uses the test codec in testcodec.py and thus also tests the\nencodings package lookup scheme.\n\nWritten by Marc-Andre Lemburg (mal@lemburg.com).\n\n(c) Copyright 2000 Guido van Rossum.\n\n'

def codec_search_function(encoding):
    if encoding == 'testcodec':
        from test import testcodec
        return tuple(testcodec.getregentry())
    return None

codecname = 'testcodec'


# --- test body ---
codecs.register(codec_search_function)
pass

assert 'abc'.encode(codecname) == b'abc'

assert 'xdef'.encode(codecname) == b'abcdef'

assert 'defx'.encode(codecname) == b'defabc'

assert 'dxf'.encode(codecname) == b'dabcf'

assert 'dxfx'.encode(codecname) == b'dabcfabc'
print("CharmapCodecTest::test_encodex: ok")
