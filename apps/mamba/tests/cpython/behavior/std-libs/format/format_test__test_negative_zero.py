# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "format"
# dimension = "behavior"
# case = "format_test__test_negative_zero"
# subject = "cpython.test_format.FormatTest.test_negative_zero"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_format.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_format.py::FormatTest::test_negative_zero
"""Auto-ported test: FormatTest::test_negative_zero (CPython 3.12 oracle)."""


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

assert f'{-0.0:.1f}' == '-0.0'

assert f'{-0.01:.1f}' == '-0.0'

assert f'{-0:.1f}' == '0.0'

assert f'{0.0:z.1f}' == '0.0'

assert f'{0.0:z6.1f}' == '   0.0'

assert f'{-1.0:z6.1f}' == '  -1.0'

assert f'{-0.0:z.1f}' == '0.0'

assert f'{0.01:z.1f}' == '0.0'

assert f'{-0:z.1f}' == '0.0'

assert f'{-0.01:z.1f}' == '0.0'

assert f'{0.0:z.2f}' == '0.00'

assert f'{-0.0:z.2f}' == '0.00'

assert f'{0.001:z.2f}' == '0.00'

assert f'{-0.001:z.2f}' == '0.00'

assert f'{0.0:z.1e}' == '0.0e+00'

assert f'{-0.0:z.1e}' == '0.0e+00'

assert f'{0.0:z.1E}' == '0.0E+00'

assert f'{-0.0:z.1E}' == '0.0E+00'

assert f'{-0.001:z.2e}' == '-1.00e-03'

assert f'{-0.001:z.2g}' == '-0.001'

assert f'{-0.001:z.2%}' == '-0.10%'

assert f'{-1e-06:z.1f}' == '0.0'

assert f'{-0.0:z.1f}' == '0.0'

assert f'{-0.0:z.1f}' == '0.0'

assert f'{-1e-06:z.2f}' == '0.00'

assert f'{-0.0:z.2f}' == '0.00'

assert f'{-0.0:z.2f}' == '0.00'

assert f'{0.09:z.1f}' == '0.1'

assert f'{-0.09:z.1f}' == '-0.1'

assert f'{-0.0: z.0f}' == ' 0'

assert f'{-0.0:+z.0f}' == '+0'

assert f'{-0.0:-z.0f}' == '0'

assert f'{-1.0: z.0f}' == '-1'

assert f'{-1.0:+z.0f}' == '-1'

assert f'{-1.0:-z.0f}' == '-1'

assert f'{0j:z.1f}' == '0.0+0.0j'

assert f'{-0j:z.1f}' == '0.0+0.0j'

assert f'{0.01j:z.1f}' == '0.0+0.0j'

assert f'{-0.01j:z.1f}' == '0.0+0.0j'

assert f'{-0.0:z>6.1f}' == 'zz-0.0'

assert f'{-0.0:z>z6.1f}' == 'zzz0.0'

assert f'{-0.0:x>z6.1f}' == 'xxx0.0'

assert f'{-0.0:🖤>z6.1f}' == '🖤🖤🖤0.0'
print("FormatTest::test_negative_zero: ok")
