# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "time_test_case__test_time_ns_type_uccc6484"
# subject = "cpython.test_time.TimeTestCase.test_time_ns_type"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import decimal
import enum
import math
import platform
import sys
import sysconfig
import time
import threading

def _bounds_checking(func):
    func((1900, 0, 1, 0, 0, 0, 0, 1, -1))
    func((1900, 12, 1, 0, 0, 0, 0, 1, -1))
    try:
        func((1900, -1, 1, 0, 0, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        func((1900, 13, 1, 0, 0, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    func((1900, 1, 0, 0, 0, 0, 0, 1, -1))
    func((1900, 1, 31, 0, 0, 0, 0, 1, -1))
    try:
        func((1900, 1, -1, 0, 0, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        func((1900, 1, 32, 0, 0, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    func((1900, 1, 1, 23, 0, 0, 0, 1, -1))
    try:
        func((1900, 1, 1, -1, 0, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        func((1900, 1, 1, 24, 0, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    func((1900, 1, 1, 0, 59, 0, 0, 1, -1))
    try:
        func((1900, 1, 1, 0, -1, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        func((1900, 1, 1, 0, 60, 0, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        func((1900, 1, 1, 0, 0, -1, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    func((1900, 1, 1, 0, 0, 60, 0, 1, -1))
    func((1900, 1, 1, 0, 0, 61, 0, 1, -1))
    try:
        func((1900, 1, 1, 0, 0, 62, 0, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    assert func((1900, 1, 1, 0, 0, 0, -1, 1, -1)) == func((1900, 1, 1, 0, 0, 0, +6, 1, -1))
    try:
        func((1900, 1, 1, 0, 0, 0, -2, 1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    func((1900, 1, 1, 0, 0, 0, 0, 0, -1))
    func((1900, 1, 1, 0, 0, 0, 0, 366, -1))
    try:
        func((1900, 1, 1, 0, 0, 0, 0, -1, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
    try:
        func((1900, 1, 1, 0, 0, 0, 0, 367, -1))
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
self_t = time.time()

def check_ns(sec, ns):
    assert isinstance(ns, int)
    sec_ns = int(sec * 1000000000.0)
    assert sec_ns - ns < 50 ** 6
check_ns(time.time(), time.time_ns())
check_ns(time.monotonic(), time.monotonic_ns())
check_ns(time.perf_counter(), time.perf_counter_ns())
check_ns(time.process_time(), time.process_time_ns())
if hasattr(time, 'thread_time'):
    check_ns(time.thread_time(), time.thread_time_ns())
if hasattr(time, 'clock_gettime'):
    check_ns(time.clock_gettime(time.CLOCK_REALTIME), time.clock_gettime_ns(time.CLOCK_REALTIME))

print("TimeTestCase::test_time_ns_type: ok")
