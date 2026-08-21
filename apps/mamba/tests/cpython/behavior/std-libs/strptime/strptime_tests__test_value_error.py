# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strptime"
# dimension = "behavior"
# case = "strptime_tests__test_value_error"
# subject = "cpython.test_strptime.StrptimeTests.test_ValueError"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strptime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strptime.py::StrptimeTests::test_ValueError
"""Auto-ported test: StrptimeTests::test_ValueError (CPython 3.12 oracle)."""


import unittest
import time
import locale
import re
import os
import platform
import sys
from test import support
from test.support import skip_if_buggy_ucrt_strfptime, run_with_locales
from datetime import date as datetime_date
import _strptime


'PyUnit testing against strptime'

libc_ver = platform.libc_ver()

if libc_ver[0] == 'glibc':
    glibc_ver = tuple(map(int, libc_ver[1].split('.')))
else:
    glibc_ver = None


# --- test body ---
def roundtrip(fmt, position, time_tuple=None):
    """Helper fxn in testing."""
    if time_tuple is None:
        time_tuple = self_time_tuple
    strf_output = time.strftime(fmt, time_tuple)
    strp_output = _strptime._strptime_time(strf_output, fmt)

    assert strp_output[position] == time_tuple[position]
    if support.verbose >= 3:
        print('testing of %r format: %r -> %r' % (fmt, strf_output, strp_output[position]))
'Create testing time tuples.'
self_time_tuple = time.localtime()

try:
    _strptime._strptime_time(data_string='%d', format='%A')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for bad_format in ('%', '% ', '%e'):
    try:
        _strptime._strptime_time('2005', bad_format)
    except ValueError:
        continue
    except Exception as err:

        raise AssertionError("'%s' raised %s, not ValueError" % (bad_format, err.__class__.__name__))
    else:

        raise AssertionError("'%s' did not raise ValueError" % bad_format)
msg_week_no_year_or_weekday = "ISO week directive '%V' must be used with the ISO year directive '%G' and a weekday directive \\('%A', '%a', '%w', or '%u'\\)."
msg_week_not_compatible = "ISO week directive '%V' is incompatible with the year directive '%Y'. Use the ISO year '%G' instead."
msg_julian_not_compatible = "Day of the year directive '%j' is not compatible with ISO year directive '%G'. Use '%Y' instead."
msg_year_no_week_or_weekday = "ISO year directive '%G' must be used with the ISO week directive '%V' and a weekday directive \\('%A', '%a', '%w', or '%u'\\)."
locale_time = _strptime.LocaleTime()
subtests = [('1999 50', '%Y %V', msg_week_no_year_or_weekday), ('1999 50 5', '%Y %V %u', msg_week_not_compatible), ('1999 51', '%G %V', msg_year_no_week_or_weekday), ('1999 {}'.format(locale_time.f_weekday[5]), '%G %A', msg_year_no_week_or_weekday), ('1999 {}'.format(locale_time.a_weekday[5]), '%G %a', msg_year_no_week_or_weekday), ('1999 5', '%G %w', msg_year_no_week_or_weekday), ('1999 5', '%G %u', msg_year_no_week_or_weekday), ('2015', '%G', msg_year_no_week_or_weekday), ('1999 256', '%G %j', msg_julian_not_compatible), ('1999 50 5 256', '%G %V %u %j', msg_julian_not_compatible), ('50', '%V', msg_week_no_year_or_weekday), ('50 5', '%V %u', msg_week_no_year_or_weekday), ('2019-00-1', '%G-%V-%u', "time data '2019-00-1' does not match format '%G-%V-%u'"), ('2019-54-1', '%G-%V-%u', "time data '2019-54-1' does not match format '%G-%V-%u'"), ('2021-53-1', '%G-%V-%u', 'Invalid week: 53')]
for data_string, format, message in subtests:
    try:
        _strptime._strptime(data_string, format)
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(message, str(_aR_e))
print("StrptimeTests::test_ValueError: ok")
