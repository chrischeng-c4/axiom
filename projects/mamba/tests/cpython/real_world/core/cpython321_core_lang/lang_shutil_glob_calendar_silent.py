# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_shutil_glob_calendar_silent"
# subject = "cpython321.lang_shutil_glob_calendar_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_shutil_glob_calendar_silent.py"
# status = "filled"
# ///
"""cpython321.lang_shutil_glob_calendar_silent: execute CPython 3.12 seed lang_shutil_glob_calendar_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `shutil.disk_usage('/').total`
# (the documented "total is the filesystem capacity in bytes" —
# mamba returns 0), `shutil.disk_usage('/').free` (the documented
# "free is the filesystem free-space in bytes" — mamba returns 0),
# `type(glob.iglob(...)).__name__` (the documented "iglob returns a
# generator that yields paths lazily" — mamba returns 'list'),
# `hasattr(iglob, '__next__')` (the documented "iglob result is an
# iterator with __next__" — mamba returns False), `len(calendar.month
# (2024, 2)) > 0` (the documented "calendar.month renders a multi-line
# string of the month" — mamba returns ''), `"February" in calendar
# .month(2024, 2)` (the documented "calendar.month header contains the
# month name" — mamba returns False), `calendar.weekheader(3)` (the
# documented "weekheader returns the three-letter day-name header
# 'Mon Tue Wed Thu Fri Sat Sun'" — mamba returns ''),
# `hasattr(calendar.Calendar(), 'iterweekdays')` (the documented
# "Calendar.iterweekdays yields weekday numbers" — mamba returns
# False), `hasattr(calendar.Calendar(), 'monthdayscalendar')` (the
# documented "Calendar.monthdayscalendar returns a list-of-weeks
# matrix" — mamba returns False), and `hasattr(calendar.TextCalendar()
# , 'formatmonth')` (the documented "TextCalendar.formatmonth returns
# the rendered month string" — mamba returns False).
# Ten-pack pinned to atomic 258.
#
# Behavioral edges that CONFORM on mamba (shutil — hasattr copy/copy2/
# copyfile/copytree/rmtree/move/which/disk_usage/make_archive/
# unpack_archive/get_terminal_size/copyfileobj/copymode/copystat/
# ignore_patterns/SameFileError/Error + which("python3") not None,
# which("fakefakefake") is None, get_terminal_size().columns is int.
# glob — hasattr glob/iglob/escape + escape("a*b?c[d]")
# == 'a[*]b[?]c[[]d]', escape("?") == '[?]', escape("a*") == 'a[*]',
# escape("[]") == '[[]]', glob('/nonexistent/*') == [], iglob has
# __iter__. fnmatch — hasattr fnmatch/fnmatchcase/filter/translate +
# fnmatch true/false cases, fnmatchcase case-sensitive, filter list
# result, translate is str, char-class [abc]/[!abc]/[a-z], empty
# pattern. calendar — hasattr month/monthrange/isleap/leapdays/
# Calendar/TextCalendar/day_name/month_name/MONDAY/SUNDAY/
# setfirstweekday/HTMLCalendar/different_locale/IllegalMonthError +
# isleap(2024/2023/2000/1900), leapdays(2000,2024)==6, monthrange
# (2024,2)==(3,29), monthrange(2023,2)==(2,28), MONDAY==0, SUNDAY==6,
# day_name[0]=='Monday', month_name[1]=='January', month_name[12]==
# 'December', Calendar() class name, weekday(2024,2,1)==3, timegm==
# 1704067200, firstweekday()==0, monthcalendar(2024,2) is list of
# len 5 containing 29) are covered in the matching pass fixture
# `test_shutil_glob_fnmatch_calendar_value_ops`.
import shutil
import glob
import calendar


_ledger: list[int] = []

# 1) shutil.disk_usage('/').total > 0 — non-empty filesystem
#    (mamba: returns 0)
assert shutil.disk_usage("/").total > 0; _ledger.append(1)

# 2) shutil.disk_usage('/').free > 0 — non-zero free space
#    (mamba: returns 0)
assert shutil.disk_usage("/").free > 0; _ledger.append(1)

# 3) type(glob.iglob(...)).__name__ == 'generator'
#    (mamba: returns 'list')
assert type(glob.iglob("/tmp/*.py")).__name__ == "generator"; _ledger.append(1)

# 4) hasattr(iglob result, '__next__')
#    (mamba: returns False)
assert hasattr(glob.iglob("/tmp/*.py"), "__next__") == True; _ledger.append(1)

# 5) len(calendar.month(2024, 2)) > 0 — non-empty render
#    (mamba: returns '')
assert len(calendar.month(2024, 2)) > 0; _ledger.append(1)

# 6) "February" in calendar.month(2024, 2)
#    (mamba: returns empty string — substring absent)
assert ("February" in calendar.month(2024, 2)) == True; _ledger.append(1)

# 7) calendar.weekheader(3) == three-letter weekday header
#    (mamba: returns '')
assert calendar.weekheader(3) == "Mon Tue Wed Thu Fri Sat Sun"; _ledger.append(1)

# 8) hasattr(calendar.Calendar(), 'iterweekdays')
#    (mamba: returns False)
assert hasattr(calendar.Calendar(), "iterweekdays") == True; _ledger.append(1)

# 9) hasattr(calendar.Calendar(), 'monthdayscalendar')
#    (mamba: returns False)
assert hasattr(calendar.Calendar(), "monthdayscalendar") == True; _ledger.append(1)

# 10) hasattr(calendar.TextCalendar(), 'formatmonth')
#     (mamba: returns False)
assert hasattr(calendar.TextCalendar(), "formatmonth") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_shutil_glob_calendar_silent {sum(_ledger)} asserts")
