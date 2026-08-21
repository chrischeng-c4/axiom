# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_format"
# subject = "cpython.test_complex.ComplexTest.test_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_complex.py::ComplexTest::test_format
"""Auto-ported test: ComplexTest::test_format (CPython 3.12 oracle)."""


import unittest
import sys
from test import support
from test.support.testcase import ComplexesAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from random import random
from math import isnan, copysign
import operator


INF = float('inf')

NAN = float('nan')

ZERO_DIVISION = ((1 + 1j, 0 + 0j), (1 + 1j, 0.0), (1 + 1j, 0), (1.0, 0 + 0j), (1, 0 + 0j))

class WithIndex:

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value

class WithFloat:

    def __init__(self, value):
        self.value = value

    def __float__(self):
        return self.value

class ComplexSubclass(complex):
    pass

class WithComplex:

    def __init__(self, value):
        self.value = value

    def __complex__(self):
        return self.value


# --- test body ---

assert format(1 + 3j, '') == str(1 + 3j)

assert format(1.5 + 3.5j, '') == str(1.5 + 3.5j)

assert format(3j, '') == str(3j)

assert format(3.2j, '') == str(3.2j)

assert format(3 + 0j, '') == str(3 + 0j)

assert format(3.2 + 0j, '') == str(3.2 + 0j)

assert format(3.2 + 0j, '-') == str(3.2 + 0j)

assert format(3.2 + 0j, '<') == str(3.2 + 0j)
z = 4 / 7.0 - 100j / 7.0

assert format(z, '') == str(z)

assert format(z, '-') == str(z)

assert format(z, '<') == str(z)

assert format(z, '10') == str(z)
z = complex(0.0, 3.0)

assert format(z, '') == str(z)

assert format(z, '-') == str(z)

assert format(z, '<') == str(z)

assert format(z, '2') == str(z)
z = complex(-0.0, 2.0)

assert format(z, '') == str(z)

assert format(z, '-') == str(z)

assert format(z, '<') == str(z)

assert format(z, '3') == str(z)

assert format(1 + 3j, 'g') == '1+3j'

assert format(3j, 'g') == '0+3j'

assert format(1.5 + 3.5j, 'g') == '1.5+3.5j'

assert format(1.5 + 3.5j, '+g') == '+1.5+3.5j'

assert format(1.5 - 3.5j, '+g') == '+1.5-3.5j'

assert format(1.5 - 3.5j, '-g') == '1.5-3.5j'

assert format(1.5 + 3.5j, ' g') == ' 1.5+3.5j'

assert format(1.5 - 3.5j, ' g') == ' 1.5-3.5j'

assert format(-1.5 + 3.5j, ' g') == '-1.5+3.5j'

assert format(-1.5 - 3.5j, ' g') == '-1.5-3.5j'

assert format(-1.5 - 3.5e-20j, 'g') == '-1.5-3.5e-20j'

assert format(-1.5 - 3.5j, 'f') == '-1.500000-3.500000j'

assert format(-1.5 - 3.5j, 'F') == '-1.500000-3.500000j'

assert format(-1.5 - 3.5j, 'e') == '-1.500000e+00-3.500000e+00j'

assert format(-1.5 - 3.5j, '.2e') == '-1.50e+00-3.50e+00j'

assert format(-1.5 - 3.5j, '.2E') == '-1.50E+00-3.50E+00j'

assert format(-15000000000.0 - 350000j, '.2G') == '-1.5E+10-3.5E+05j'

assert format(1.5 + 3j, '<20g') == '1.5+3j              '

assert format(1.5 + 3j, '*<20g') == '1.5+3j**************'

assert format(1.5 + 3j, '>20g') == '              1.5+3j'

assert format(1.5 + 3j, '^20g') == '       1.5+3j       '

assert format(1.5 + 3j, '<20') == '(1.5+3j)            '

assert format(1.5 + 3j, '>20') == '            (1.5+3j)'

assert format(1.5 + 3j, '^20') == '      (1.5+3j)      '

assert format(1.123 - 3.123j, '^20.2') == '     (1.1-3.1j)     '

assert format(1.5 + 3j, '20.2f') == '          1.50+3.00j'

assert format(1.5 + 3j, '>20.2f') == '          1.50+3.00j'

assert format(1.5 + 3j, '<20.2f') == '1.50+3.00j          '

assert format(1.5e+20 + 3j, '<20.2f') == '150000000000000000000.00+3.00j'

assert format(1.5e+20 + 3j, '>40.2f') == '          150000000000000000000.00+3.00j'

assert format(1.5e+20 + 3j, '^40,.2f') == '  150,000,000,000,000,000,000.00+3.00j  '

assert format(1.5e+21 + 3j, '^40,.2f') == ' 1,500,000,000,000,000,000,000.00+3.00j '

assert format(1.5e+21 + 3000j, ',.2f') == '1,500,000,000,000,000,000,000.00+3,000.00j'

assert format(1 + 1j, '.0e') == '1e+00+1e+00j'

assert format(1 + 1j, '#.0e') == '1.e+00+1.e+00j'

assert format(1 + 1j, '.0f') == '1+1j'

assert format(1 + 1j, '#.0f') == '1.+1.j'

assert format(1.1 + 1.1j, 'g') == '1.1+1.1j'

assert format(1.1 + 1.1j, '#g') == '1.10000+1.10000j'

assert format(1 + 1j, '.1e') == '1.0e+00+1.0e+00j'

assert format(1 + 1j, '#.1e') == '1.0e+00+1.0e+00j'

assert format(1 + 1j, '.1f') == '1.0+1.0j'

assert format(1 + 1j, '#.1f') == '1.0+1.0j'

assert format(-1.5 + 0.5j, '#f') == '-1.500000+0.500000j'

assert format(-1.5 + 0.5j, '#.0f') == '-2.+0.j'

assert format(-1.5 + 0.5j, '#e') == '-1.500000e+00+5.000000e-01j'

assert format(-1.5 + 0.5j, '#.0e') == '-2.e+00+5.e-01j'

assert format(-1.5 + 0.5j, '#g') == '-1.50000+0.500000j'

assert format(-1.5 + 0.5j, '.0g') == '-2+0.5j'

assert format(-1.5 + 0.5j, '#.0g') == '-2.+0.5j'

try:
    (1.5 + 0.5j).__format__('010f')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (1.5 + 3j).__format__('=20')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for t in 'bcdoxX':

    try:
        (1.5 + 0.5j).__format__(t)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert '*{0:.3f}*'.format(3.14159 + 2.71828j) == '*3.142+2.718j*'

assert format(complex(NAN, NAN), 'f') == 'nan+nanj'

assert format(complex(1, NAN), 'f') == '1.000000+nanj'

assert format(complex(NAN, 1), 'f') == 'nan+1.000000j'

assert format(complex(NAN, -1), 'f') == 'nan-1.000000j'

assert format(complex(NAN, NAN), 'F') == 'NAN+NANj'

assert format(complex(1, NAN), 'F') == '1.000000+NANj'

assert format(complex(NAN, 1), 'F') == 'NAN+1.000000j'

assert format(complex(NAN, -1), 'F') == 'NAN-1.000000j'

assert format(complex(INF, INF), 'f') == 'inf+infj'

assert format(complex(1, INF), 'f') == '1.000000+infj'

assert format(complex(INF, 1), 'f') == 'inf+1.000000j'

assert format(complex(INF, -1), 'f') == 'inf-1.000000j'

assert format(complex(INF, INF), 'F') == 'INF+INFj'

assert format(complex(1, INF), 'F') == '1.000000+INFj'

assert format(complex(INF, 1), 'F') == 'INF+1.000000j'

assert format(complex(INF, -1), 'F') == 'INF-1.000000j'
print("ComplexTest::test_format: ok")
