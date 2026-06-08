# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "format"
# dimension = "behavior"
# case = "format_test__test_common_format"
# subject = "cpython.test_format.FormatTest.test_common_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_format.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_format.py::FormatTest::test_common_format
"""Auto-ported test: FormatTest::test_common_format (CPython 3.12 oracle)."""


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
testcommon('%%', (), '%')
testcommon('%.1d', (1,), '1')
testcommon('%.*d', (sys.maxsize, 1), overflowok=True)
testcommon('%.100d', (1,), '0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001', overflowok=True)
testcommon('%#.117x', (1,), '0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001', overflowok=True)
testcommon('%#.118x', (1,), '0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001', overflowok=True)
testcommon('%f', (1.0,), '1.000000')
testcommon('%#.*g', (109, -1e+49 / 3.0))
testcommon('%#.*g', (110, -1e+49 / 3.0))
testcommon('%#.*g', (110, -1e+100 / 3.0))
testcommon('%12.*f', (123456, 1.0))
testcommon('%#.*g', (110, -1e+100 / 3.0))
testcommon('%#.*G', (110, -1e+100 / 3.0))
testcommon('%#.*f', (110, -1e+100 / 3.0))
testcommon('%#.*F', (110, -1e+100 / 3.0))
testcommon('%x', 10, 'a')
testcommon('%x', 100000000000, '174876e800')
testcommon('%o', 10, '12')
testcommon('%o', 100000000000, '1351035564000')
testcommon('%d', 10, '10')
testcommon('%d', 100000000000, '100000000000')
big = 123456789012345678901234567890
testcommon('%d', big, '123456789012345678901234567890')
testcommon('%d', -big, '-123456789012345678901234567890')
testcommon('%5d', -big, '-123456789012345678901234567890')
testcommon('%31d', -big, '-123456789012345678901234567890')
testcommon('%32d', -big, ' -123456789012345678901234567890')
testcommon('%-32d', -big, '-123456789012345678901234567890 ')
testcommon('%032d', -big, '-0123456789012345678901234567890')
testcommon('%-032d', -big, '-123456789012345678901234567890 ')
testcommon('%034d', -big, '-000123456789012345678901234567890')
testcommon('%034d', big, '0000123456789012345678901234567890')
testcommon('%0+34d', big, '+000123456789012345678901234567890')
testcommon('%+34d', big, '   +123456789012345678901234567890')
testcommon('%34d', big, '    123456789012345678901234567890')
testcommon('%.2d', big, '123456789012345678901234567890')
testcommon('%.30d', big, '123456789012345678901234567890')
testcommon('%.31d', big, '0123456789012345678901234567890')
testcommon('%32.31d', big, ' 0123456789012345678901234567890')
testcommon('%d', float(big), '123456________________________', 6)
big = 1375488932362216742658885
testcommon('%x', big, '1234567890abcdef12345')
testcommon('%x', -big, '-1234567890abcdef12345')
testcommon('%5x', -big, '-1234567890abcdef12345')
testcommon('%22x', -big, '-1234567890abcdef12345')
testcommon('%23x', -big, ' -1234567890abcdef12345')
testcommon('%-23x', -big, '-1234567890abcdef12345 ')
testcommon('%023x', -big, '-01234567890abcdef12345')
testcommon('%-023x', -big, '-1234567890abcdef12345 ')
testcommon('%025x', -big, '-0001234567890abcdef12345')
testcommon('%025x', big, '00001234567890abcdef12345')
testcommon('%0+25x', big, '+0001234567890abcdef12345')
testcommon('%+25x', big, '   +1234567890abcdef12345')
testcommon('%25x', big, '    1234567890abcdef12345')
testcommon('%.2x', big, '1234567890abcdef12345')
testcommon('%.21x', big, '1234567890abcdef12345')
testcommon('%.22x', big, '01234567890abcdef12345')
testcommon('%23.22x', big, ' 01234567890abcdef12345')
testcommon('%-23.22x', big, '01234567890abcdef12345 ')
testcommon('%X', big, '1234567890ABCDEF12345')
testcommon('%#X', big, '0X1234567890ABCDEF12345')
testcommon('%#x', big, '0x1234567890abcdef12345')
testcommon('%#x', -big, '-0x1234567890abcdef12345')
testcommon('%#27x', big, '    0x1234567890abcdef12345')
testcommon('%#-27x', big, '0x1234567890abcdef12345    ')
testcommon('%#027x', big, '0x00001234567890abcdef12345')
testcommon('%#.23x', big, '0x001234567890abcdef12345')
testcommon('%#.23x', -big, '-0x001234567890abcdef12345')
testcommon('%#27.23x', big, '  0x001234567890abcdef12345')
testcommon('%#-27.23x', big, '0x001234567890abcdef12345  ')
testcommon('%#027.23x', big, '0x00001234567890abcdef12345')
testcommon('%#+.23x', big, '+0x001234567890abcdef12345')
testcommon('%# .23x', big, ' 0x001234567890abcdef12345')
testcommon('%#+.23X', big, '+0X001234567890ABCDEF12345')
testcommon('%#+027.23X', big, '+0X0001234567890ABCDEF12345')
testcommon('%# 027.23X', big, ' 0X0001234567890ABCDEF12345')
testcommon('%#+27.23X', big, ' +0X001234567890ABCDEF12345')
testcommon('%#-+27.23x', big, '+0x001234567890abcdef12345 ')
testcommon('%#- 27.23x', big, ' 0x001234567890abcdef12345 ')
big = 12935167030485801517351291832
testcommon('%o', big, '12345670123456701234567012345670')
testcommon('%o', -big, '-12345670123456701234567012345670')
testcommon('%5o', -big, '-12345670123456701234567012345670')
testcommon('%33o', -big, '-12345670123456701234567012345670')
testcommon('%34o', -big, ' -12345670123456701234567012345670')
testcommon('%-34o', -big, '-12345670123456701234567012345670 ')
testcommon('%034o', -big, '-012345670123456701234567012345670')
testcommon('%-034o', -big, '-12345670123456701234567012345670 ')
testcommon('%036o', -big, '-00012345670123456701234567012345670')
testcommon('%036o', big, '000012345670123456701234567012345670')
testcommon('%0+36o', big, '+00012345670123456701234567012345670')
testcommon('%+36o', big, '   +12345670123456701234567012345670')
testcommon('%36o', big, '    12345670123456701234567012345670')
testcommon('%.2o', big, '12345670123456701234567012345670')
testcommon('%.32o', big, '12345670123456701234567012345670')
testcommon('%.33o', big, '012345670123456701234567012345670')
testcommon('%34.33o', big, ' 012345670123456701234567012345670')
testcommon('%-34.33o', big, '012345670123456701234567012345670 ')
testcommon('%o', big, '12345670123456701234567012345670')
testcommon('%#o', big, '0o12345670123456701234567012345670')
testcommon('%#o', -big, '-0o12345670123456701234567012345670')
testcommon('%#38o', big, '    0o12345670123456701234567012345670')
testcommon('%#-38o', big, '0o12345670123456701234567012345670    ')
testcommon('%#038o', big, '0o000012345670123456701234567012345670')
testcommon('%#.34o', big, '0o0012345670123456701234567012345670')
testcommon('%#.34o', -big, '-0o0012345670123456701234567012345670')
testcommon('%#38.34o', big, '  0o0012345670123456701234567012345670')
testcommon('%#-38.34o', big, '0o0012345670123456701234567012345670  ')
testcommon('%#038.34o', big, '0o000012345670123456701234567012345670')
testcommon('%#+.34o', big, '+0o0012345670123456701234567012345670')
testcommon('%# .34o', big, ' 0o0012345670123456701234567012345670')
testcommon('%#+38.34o', big, ' +0o0012345670123456701234567012345670')
testcommon('%#-+38.34o', big, '+0o0012345670123456701234567012345670 ')
testcommon('%#- 38.34o', big, ' 0o0012345670123456701234567012345670 ')
testcommon('%#+038.34o', big, '+0o00012345670123456701234567012345670')
testcommon('%# 038.34o', big, ' 0o00012345670123456701234567012345670')
testcommon('%.33o', big, '012345670123456701234567012345670')
testcommon('%#.33o', big, '0o012345670123456701234567012345670')
testcommon('%#.32o', big, '0o12345670123456701234567012345670')
testcommon('%035.33o', big, '00012345670123456701234567012345670')
testcommon('%0#35.33o', big, '0o012345670123456701234567012345670')
testcommon('%d', 42, '42')
testcommon('%d', -42, '-42')
testcommon('%d', 42.0, '42')
testcommon('%#x', 1, '0x1')
testcommon('%#X', 1, '0X1')
testcommon('%#o', 1, '0o1')
testcommon('%#o', 0, '0o0')
testcommon('%o', 0, '0')
testcommon('%d', 0, '0')
testcommon('%#x', 0, '0x0')
testcommon('%#X', 0, '0X0')
testcommon('%x', 66, '42')
testcommon('%x', -66, '-42')
testcommon('%o', 34, '42')
testcommon('%o', -34, '-42')
testcommon('%g', 1.1, '1.1')
testcommon('%#g', 1.1, '1.10000')
if verbose:
    print('Testing exceptions')
test_exc_common('%', (), ValueError, 'incomplete format')
test_exc_common('% %s', 1, ValueError, "unsupported format character '%' (0x25) at index 2")
test_exc_common('%d', '1', TypeError, '%d format: a real number is required, not str')
test_exc_common('%d', b'1', TypeError, '%d format: a real number is required, not bytes')
test_exc_common('%x', '1', TypeError, '%x format: an integer is required, not str')
test_exc_common('%x', 3.14, TypeError, '%x format: an integer is required, not float')
print("FormatTest::test_common_format: ok")
