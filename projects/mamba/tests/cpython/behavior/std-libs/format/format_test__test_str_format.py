# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "format"
# dimension = "behavior"
# case = "format_test__test_str_format"
# subject = "cpython.test_format.FormatTest.test_str_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_format.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_format.py::FormatTest::test_str_format
"""Auto-ported test: FormatTest::test_str_format (CPython 3.12 oracle)."""


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
testformat('%r', '\u0378', "'\\u0378'")
testformat('%a', '\u0378', "'\\u0378'")
testformat('%r', 'ʹ', "'ʹ'")
testformat('%a', 'ʹ', "'\\u0374'")
if verbose:
    print('Testing exceptions')
test_exc('abc %b', 1, ValueError, "unsupported format character 'b' (0x62) at index 5")
test_exc('%g', '1', TypeError, 'must be real number, not str')
test_exc('no format', '1', TypeError, 'not all arguments converted during string formatting')
test_exc('%c', -1, OverflowError, '%c arg not in range(0x110000)')
test_exc('%c', sys.maxunicode + 1, OverflowError, '%c arg not in range(0x110000)')
test_exc('%c', 3.14, TypeError, '%c requires int or char')
test_exc('%c', 'ab', TypeError, '%c requires int or char')
test_exc('%c', b'x', TypeError, '%c requires int or char')
if maxsize == 2 ** 31 - 1:
    try:
        '%*d' % (maxsize, -127)
    except MemoryError:
        pass
    else:
        raise TestFailed('"%*d"%(maxsize, -127) should fail')
print("FormatTest::test_str_format: ok")
