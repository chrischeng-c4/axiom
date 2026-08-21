# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strftime"
# dimension = "behavior"
# case = "strftime_test__test_strftime"
# subject = "cpython.test_strftime.StrftimeTest.test_strftime"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_strftime.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_strftime.py::StrftimeTest::test_strftime
"""Auto-ported test: StrftimeTest::test_strftime (CPython 3.12 oracle)."""


import calendar
from locale import LC_TIME, setlocale
import re
from test import support
import time


def fixasctime(text):
    if text[8] == " ":
        text = text[:8] + "0" + text[9:]
    return text


def escapestr(text, ampm):
    new_text = re.escape(text)
    new_text = new_text.replace(re.escape(ampm), ampm)
    new_text = new_text.replace(r"\%", "%")
    new_text = new_text.replace(r"\:", ":")
    new_text = new_text.replace(r"\?", "?")
    return new_text


def update_variables(timestamp):
    gmt = time.gmtime(timestamp)
    now = time.localtime(timestamp)
    if now[3] < 12:
        ampm = "(AM|am)"
    else:
        ampm = "(PM|pm)"
    jan1 = time.localtime(time.mktime((now[0], 1, 1, 0, 0, 0, 0, 1, 0)))
    try:
        if now[8]:
            tz = time.tzname[1]
        else:
            tz = time.tzname[0]
    except AttributeError:
        tz = ""
    if now[3] > 12:
        clock12 = now[3] - 12
    elif now[3] > 0:
        clock12 = now[3]
    else:
        clock12 = 12
    return gmt, now, ampm, jan1, tz, clock12


def strftest1(timestamp):
    _gmt, now, ampm, jan1, _tz, clock12 = update_variables(timestamp)
    expectations = (
        ("%a", calendar.day_abbr[now[6]], "abbreviated weekday name"),
        ("%A", calendar.day_name[now[6]], "full weekday name"),
        ("%b", calendar.month_abbr[now[1]], "abbreviated month name"),
        ("%B", calendar.month_name[now[1]], "full month name"),
        ("%d", "%02d" % now[2], "day of month as number (00-31)"),
        ("%H", "%02d" % now[3], "hour (00-23)"),
        ("%I", "%02d" % clock12, "hour (01-12)"),
        ("%j", "%03d" % now[7], "julian day (001-366)"),
        ("%m", "%02d" % now[1], "month as number (01-12)"),
        ("%M", "%02d" % now[4], "minute, (00-59)"),
        ("%p", ampm, "AM or PM as appropriate"),
        ("%S", "%02d" % now[5], "seconds of current time (00-60)"),
        ("%U", "%02d" % ((now[7] + jan1[6]) // 7), "week number of the year"),
        ("%w", "0?%d" % ((1 + now[6]) % 7), "weekday as a number"),
        ("%W", "%02d" % ((now[7] + (jan1[6] - 1) % 7) // 7), "week number"),
        ("%X", "%02d:%02d:%02d" % (now[3], now[4], now[5]), "%H:%M:%S"),
        ("%y", "%02d" % (now[0] % 100), "year without century"),
        ("%Y", "%d" % now[0], "year with century"),
        ("%%", "%", "single percent sign"),
    )

    for fmt, expected, desc in expectations:
        try:
            result = time.strftime(fmt, now)
        except ValueError as error:
            raise AssertionError(f"strftime {fmt!r} format gave error: {error}") from error
        if re.match(escapestr(expected, ampm), result):
            continue
        if not result or result[0] == "%":
            raise AssertionError(f"strftime does not support standard {fmt!r} format ({desc})")
        raise AssertionError(f"Conflict for {fmt} ({desc}): expected {expected}, got {result}")


def strftest2(timestamp):
    nowsecs = str(int(timestamp))[:-1]
    _gmt, now, ampm, _jan1, tz, clock12 = update_variables(timestamp)
    nonstandard_expectations = (
        ("%c", fixasctime(time.asctime(now)), "near-asctime() format"),
        ("%x", "%02d/%02d/%02d" % (now[1], now[2], (now[0] % 100)), "%m/%d/%y"),
        ("%Z", "%s" % tz, "time zone name"),
        ("%D", "%02d/%02d/%02d" % (now[1], now[2], (now[0] % 100)), "mm/dd/yy"),
        ("%e", "%2d" % now[2], "day of month as number, blank padded"),
        ("%h", calendar.month_abbr[now[1]], "abbreviated month name"),
        ("%k", "%2d" % now[3], "hour, blank padded"),
        ("%n", "\n", "newline character"),
        ("%r", "%02d:%02d:%02d %s" % (clock12, now[4], now[5], ampm), "%I:%M:%S %p"),
        ("%R", "%02d:%02d" % (now[3], now[4]), "%H:%M"),
        ("%s", nowsecs, "seconds since the Epoch in UCT"),
        ("%t", "\t", "tab character"),
        ("%T", "%02d:%02d:%02d" % (now[3], now[4], now[5]), "%H:%M:%S"),
        ("%3y", "%03d" % (now[0] % 100), "year without century rendered using fieldwidth"),
    )

    for fmt, expected, _desc in nonstandard_expectations:
        try:
            result = time.strftime(fmt, now)
        except ValueError:
            continue
        if re.match(escapestr(expected, ampm), result):
            continue


saved_locale = setlocale(LC_TIME)
setlocale(LC_TIME, "C")
try:
    current = time.time()
    strftest1(current)
    strftest2(current)

    for j in range(-5, 5):
        for i in range(25):
            arg = current + (i + j * 100) * 23 * 3603
            strftest1(arg)
            strftest2(arg)
finally:
    setlocale(LC_TIME, saved_locale)

print("StrftimeTest::test_strftime: ok")
