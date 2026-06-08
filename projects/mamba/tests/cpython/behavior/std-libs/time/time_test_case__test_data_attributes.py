# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_test_case__test_data_attributes"
# subject = "cpython.test_time.TimeTestCase.test_data_attributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_time.py::TimeTestCase::test_data_attributes
"""Auto-ported test: TimeTestCase::test_data_attributes (CPython 3.12 oracle)."""


from test import support
from test.support import warnings_helper
import decimal
import enum
import math
import platform
import sys
import sysconfig
import time
import threading
import unittest
from test.support import skip_if_buggy_ucrt_strfptime, SuppressCrashReport


try:
    import _testcapi
except ImportError:
    _testcapi = None

SIZEOF_INT = sysconfig.get_config_var('SIZEOF_INT') or 4

TIME_MAXYEAR = (1 << 8 * SIZEOF_INT - 1) - 1

TIME_MINYEAR = -TIME_MAXYEAR - 1 + 1900

SEC_TO_US = 10 ** 6

US_TO_NS = 10 ** 3

MS_TO_NS = 10 ** 6

SEC_TO_NS = 10 ** 9

NS_TO_SEC = 10 ** 9

class _PyTime(enum.IntEnum):
    ROUND_FLOOR = 0
    ROUND_CEILING = 1
    ROUND_HALF_EVEN = 2
    ROUND_UP = 3

_PyTime_MIN = -2 ** 63

_PyTime_MAX = 2 ** 63 - 1

ROUNDING_MODES = ((_PyTime.ROUND_FLOOR, decimal.ROUND_FLOOR), (_PyTime.ROUND_CEILING, decimal.ROUND_CEILING), (_PyTime.ROUND_HALF_EVEN, decimal.ROUND_HALF_EVEN), (_PyTime.ROUND_UP, decimal.ROUND_UP))

@unittest.skipIf(_testcapi is None, 'need the _testcapi module')
class CPyTimeTestCase:
    """
    Base class to test the C _PyTime_t API.
    """
    OVERFLOW_SECONDS = None

    def setUp(self):
        from _testcapi import SIZEOF_TIME_T
        bits = SIZEOF_TIME_T * 8 - 1
        self.time_t_min = -2 ** bits
        self.time_t_max = 2 ** bits - 1

    def time_t_filter(self, seconds):
        return self.time_t_min <= seconds <= self.time_t_max

    def _rounding_values(self, use_float):
        """Build timestamps used to test rounding."""
        units = [1, US_TO_NS, MS_TO_NS, SEC_TO_NS]
        if use_float:
            units.append(0.001)
        values = (1, 2, 5, 7, 123, 456, 1234, 9, 99, 999, 9999, 99999, 999999, 499, 500, 501, 1499, 1500, 1501, 2500, 3500, 4500)
        ns_timestamps = [0]
        for unit in units:
            for value in values:
                ns = value * unit
                ns_timestamps.extend((-ns, ns))
        for pow2 in (0, 5, 10, 15, 22, 23, 24, 30, 33):
            ns = 2 ** pow2 * SEC_TO_NS
            ns_timestamps.extend((-ns - 1, -ns, -ns + 1, ns - 1, ns, ns + 1))
        for seconds in (_testcapi.INT_MIN, _testcapi.INT_MAX):
            ns_timestamps.append(seconds * SEC_TO_NS)
        if use_float:
            for pow2 in (3, 7, 10, 15):
                ns = 2.0 ** (-pow2)
                ns_timestamps.extend((-ns, ns))
        ns = 2 ** 63 // SEC_TO_NS * SEC_TO_NS
        ns_timestamps.extend((-ns, ns))
        return ns_timestamps

    def _check_rounding(self, pytime_converter, expected_func, use_float, unit_to_sec, value_filter=None):

        def convert_values(ns_timestamps):
            if use_float:
                unit_to_ns = SEC_TO_NS / float(unit_to_sec)
                values = [ns / unit_to_ns for ns in ns_timestamps]
            else:
                unit_to_ns = SEC_TO_NS // unit_to_sec
                values = [ns // unit_to_ns for ns in ns_timestamps]
            if value_filter:
                values = filter(value_filter, values)
            return sorted(set(values))
        ns_timestamps = self._rounding_values(use_float)
        valid_values = convert_values(ns_timestamps)
        for time_rnd, decimal_rnd in ROUNDING_MODES:
            with decimal.localcontext() as context:
                context.rounding = decimal_rnd
                for value in valid_values:
                    debug_info = {'value': value, 'rounding': decimal_rnd}
                    try:
                        result = pytime_converter(value, time_rnd)
                        expected = expected_func(value)
                    except Exception:
                        self.fail('Error on timestamp conversion: %s' % debug_info)
                    self.assertEqual(result, expected, debug_info)
        ns = self.OVERFLOW_SECONDS * SEC_TO_NS
        ns_timestamps = (-ns, ns)
        overflow_values = convert_values(ns_timestamps)
        for time_rnd, _ in ROUNDING_MODES:
            for value in overflow_values:
                debug_info = {'value': value, 'rounding': time_rnd}
                with self.assertRaises(OverflowError, msg=debug_info):
                    pytime_converter(value, time_rnd)

    def check_int_rounding(self, pytime_converter, expected_func, unit_to_sec=1, value_filter=None):
        self._check_rounding(pytime_converter, expected_func, False, unit_to_sec, value_filter)

    def check_float_rounding(self, pytime_converter, expected_func, unit_to_sec=1, value_filter=None):
        self._check_rounding(pytime_converter, expected_func, True, unit_to_sec, value_filter)

    def decimal_round(self, x):
        d = decimal.Decimal(x)
        d = d.quantize(1)
        return int(d)


# --- test body ---
self_t = time.time()
time.altzone
time.daylight
time.timezone
time.tzname
print("TimeTestCase::test_data_attributes: ok")
