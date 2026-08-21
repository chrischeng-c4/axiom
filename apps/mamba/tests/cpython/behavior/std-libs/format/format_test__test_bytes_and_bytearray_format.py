# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "format"
# dimension = "behavior"
# case = "format_test__test_bytes_and_bytearray_format"
# subject = "cpython.test_format.FormatTest.test_bytes_and_bytearray_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_format.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_format.py::FormatTest::test_bytes_and_bytearray_format
"""Auto-ported test: FormatTest::test_bytes_and_bytearray_format (CPython 3.12 oracle)."""


from test.support import verbose, TestFailed
import locale
import sys
import re
import test.support as support
import unittest


maxsize = support.MAX_Py_ssize_t

def testformat(formatstr, args, output=None, limit=None, overflowok=False):
    if verbose:
        if output:
            print('{!a} % {!a} =? {!a} ...'.format(formatstr, args, output), end=' ')
        else:
            print('{!a} % {!a} works? ...'.format(formatstr, args), end=' ')
    try:
        result = formatstr % args
    except OverflowError:
        if not overflowok:
            raise
        if verbose:
            print('overflow (this is fine)')
    else:
        if output and limit is None and (result != output):
            if verbose:
                print('no')
            raise AssertionError('%r %% %r == %r != %r' % (formatstr, args, result, output))
        elif output and limit is not None and (len(result) != len(output) or result[:limit] != output[:limit]):
            if verbose:
                print('no')
            print('%s %% %s == %s != %s' % (repr(formatstr), repr(args), repr(result), repr(output)))
        elif verbose:
            print('yes')

def testcommon(formatstr, args, output=None, limit=None, overflowok=False):
    if isinstance(formatstr, str):
        testformat(formatstr, args, output, limit, overflowok)
        b_format = formatstr.encode('ascii')
    else:
        b_format = formatstr
    ba_format = bytearray(b_format)
    b_args = []
    if not isinstance(args, tuple):
        args = (args,)
    b_args = tuple(args)
    if output is None:
        b_output = ba_output = None
    else:
        if isinstance(output, str):
            b_output = output.encode('ascii')
        else:
            b_output = output
        ba_output = bytearray(b_output)
    testformat(b_format, b_args, b_output, limit, overflowok)
    testformat(ba_format, b_args, ba_output, limit, overflowok)

def test_exc(formatstr, args, exception, excmsg):
    try:
        testformat(formatstr, args)
    except exception as exc:
        if str(exc) == excmsg:
            if verbose:
                print('yes')
        else:
            if verbose:
                print('no')
            print('Unexpected ', exception, ':', repr(str(exc)))
    except:
        if verbose:
            print('no')
        print('Unexpected exception')
        raise
    else:
        raise TestFailed('did not get expected exception: %s' % excmsg)

def test_exc_common(formatstr, args, exception, excmsg):
    test_exc(formatstr, args, exception, excmsg)
    test_exc(formatstr.encode('ascii'), args, exception, excmsg)


# --- test body ---
testcommon(b'%c', 7, b'\x07')
testcommon(b'%c', b'Z', b'Z')
testcommon(b'%c', bytearray(b'Z'), b'Z')
testcommon(b'%5c', 65, b'    A')
testcommon(b'%-5c', 65, b'A    ')

class FakeBytes(object):

    def __bytes__(self):
        return b'123'
fb = FakeBytes()
testcommon(b'%b', b'abc', b'abc')
testcommon(b'%b', bytearray(b'def'), b'def')
testcommon(b'%b', fb, b'123')
testcommon(b'%b', memoryview(b'abc'), b'abc')
testcommon(b'%s', b'abc', b'abc')
testcommon(b'%s', bytearray(b'def'), b'def')
testcommon(b'%s', fb, b'123')
testcommon(b'%s', memoryview(b'abc'), b'abc')
testcommon(b'%a', 3.14, b'3.14')
testcommon(b'%a', b'ghi', b"b'ghi'")
testcommon(b'%a', 'jkl', b"'jkl'")
testcommon(b'%a', 'Մ', b"'\\u0544'")
testcommon(b'%r', 3.14, b'3.14')
testcommon(b'%r', b'ghi', b"b'ghi'")
testcommon(b'%r', 'jkl', b"'jkl'")
testcommon(b'%r', 'Մ', b"'\\u0544'")
if verbose:
    print('Testing exceptions')
test_exc(b'%g', '1', TypeError, 'float argument required, not str')
test_exc(b'%g', b'1', TypeError, 'float argument required, not bytes')
test_exc(b'no format', 7, TypeError, 'not all arguments converted during bytes formatting')
test_exc(b'no format', b'1', TypeError, 'not all arguments converted during bytes formatting')
test_exc(b'no format', bytearray(b'1'), TypeError, 'not all arguments converted during bytes formatting')
test_exc(b'%c', -1, OverflowError, '%c arg not in range(256)')
test_exc(b'%c', 256, OverflowError, '%c arg not in range(256)')
test_exc(b'%c', 2 ** 128, OverflowError, '%c arg not in range(256)')
test_exc(b'%c', b'Za', TypeError, '%c requires an integer in range(256) or a single byte')
test_exc(b'%c', 'Y', TypeError, '%c requires an integer in range(256) or a single byte')
test_exc(b'%c', 3.14, TypeError, '%c requires an integer in range(256) or a single byte')
test_exc(b'%b', 'Xc', TypeError, "%b requires a bytes-like object, or an object that implements __bytes__, not 'str'")
test_exc(b'%s', 'Wd', TypeError, "%b requires a bytes-like object, or an object that implements __bytes__, not 'str'")
if maxsize == 2 ** 31 - 1:
    try:
        '%*d' % (maxsize, -127)
    except MemoryError:
        pass
    else:
        raise TestFailed('"%*d"%(maxsize, -127) should fail')
print("FormatTest::test_bytes_and_bytearray_format: ok")
