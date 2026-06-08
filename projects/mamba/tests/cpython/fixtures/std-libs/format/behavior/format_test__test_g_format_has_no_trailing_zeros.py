# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "format"
# dimension = "behavior"
# case = "format_test__test_g_format_has_no_trailing_zeros"
# subject = "cpython.test_format.FormatTest.test_g_format_has_no_trailing_zeros"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_format.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_format.py::FormatTest::test_g_format_has_no_trailing_zeros
"""Auto-ported test: FormatTest::test_g_format_has_no_trailing_zeros (CPython 3.12 oracle)."""


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

assert '%.3g' % 1505.0 == '1.5e+03'

assert '%#.3g' % 1505.0 == '1.50e+03'

assert format(1505.0, '.3g') == '1.5e+03'

assert format(1505.0, '#.3g') == '1.50e+03'

assert format(12300050.0, '.6g') == '1.23e+07'

assert format(12300050.0, '#.6g') == '1.23000e+07'
print("FormatTest::test_g_format_has_no_trailing_zeros: ok")
