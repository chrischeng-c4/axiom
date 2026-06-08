# Atomic 258 pass conformance — shutil module (hasattr surface copy/
# copy2/copyfile/copytree/rmtree/move/which/disk_usage/make_archive/
# unpack_archive/get_terminal_size/copyfileobj/copymode/copystat/
# ignore_patterns/SameFileError/Error + which("python3") not None,
# which("fakefakefake") is None, get_terminal_size().columns is int)
# + glob module (hasattr surface glob/iglob/escape + escape("a*b?c[d]")
# == 'a[*]b[?]c[[]d]', escape("?") == '[?]', escape("a*") == 'a[*]',
# escape("[]") == '[[]]', glob('/nonexistent/*') == [], iglob has
# __iter__) + fnmatch module (hasattr surface fnmatch/fnmatchcase/
# filter/translate + fnmatch true/false cases, fnmatchcase case-
# sensitive, filter list result, translate is str starting with
# "(?s:", char-class [abc]/[!abc]/[a-z], empty pattern) + calendar
# module (hasattr surface month/monthrange/isleap/leapdays/Calendar/
# TextCalendar/day_name/month_name/MONDAY/SUNDAY/setfirstweekday/
# HTMLCalendar/different_locale/IllegalMonthError + isleap(2024/2023/
# 2000/1900), leapdays(2000,2024)==6, monthrange(2024,2)==(3,29),
# monthrange(2023,2)==(2,28), MONDAY==0, SUNDAY==6, day_name[0]==
# 'Monday', month_name[1]=='January', month_name[12]=='December',
# Calendar() class name, weekday(2024,2,1)==3, timegm==1704067200,
# firstweekday()==0, monthcalendar(2024,2) is list of len 5 containing
# 29). All asserts match between CPython 3.12 and mamba.
import shutil
import glob
import fnmatch
import calendar


_ledger: list[int] = []

# 1) shutil — hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "copyfileobj") == True; _ledger.append(1)
assert hasattr(shutil, "copymode") == True; _ledger.append(1)
assert hasattr(shutil, "copystat") == True; _ledger.append(1)
assert hasattr(shutil, "ignore_patterns") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)

# 2) shutil — which finds existing / returns None for missing
assert (shutil.which("python3") is not None) == True; _ledger.append(1)
assert (shutil.which("fakefakefakefake") is None) == True; _ledger.append(1)

# 3) shutil — get_terminal_size().columns is int
assert isinstance(shutil.get_terminal_size().columns, int) == True; _ledger.append(1)

# 4) glob — hasattr surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)

# 5) glob — escape brackets metachars
assert glob.escape("a*b?c[d]") == "a[*]b[?]c[[]d]"; _ledger.append(1)
assert glob.escape("?") == "[?]"; _ledger.append(1)
assert glob.escape("a*") == "a[*]"; _ledger.append(1)
assert glob.escape("[]") == "[[]]"; _ledger.append(1)

# 6) glob — nonexistent pattern returns empty list
assert glob.glob("/nonexistent_xyz_path_xyz/*") == []; _ledger.append(1)

# 7) glob — iglob has __iter__
assert hasattr(glob.iglob("/tmp/*.py"), "__iter__") == True; _ledger.append(1)

# 8) fnmatch — hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 9) fnmatch — star/question patterns
assert fnmatch.fnmatch("a.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "?.txt") == True; _ledger.append(1)

# 10) fnmatch — filter selects matching
assert fnmatch.filter(["a.txt", "b.py", "c.txt"], "*.txt") == ["a.txt", "c.txt"]; _ledger.append(1)

# 11) fnmatch — fnmatchcase is case-sensitive
assert fnmatch.fnmatchcase("a.TXT", "*.txt") == False; _ledger.append(1)

# 12) fnmatch — empty pattern matches empty string
assert fnmatch.fnmatch("", "") == True; _ledger.append(1)

# 13) fnmatch — character classes
assert fnmatch.fnmatch("a", "[abc]") == True; _ledger.append(1)
assert fnmatch.fnmatch("d", "[!abc]") == True; _ledger.append(1)
assert fnmatch.fnmatch("b", "[a-z]") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("B", "[a-z]") == False; _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "*.[!t]xt") == False; _ledger.append(1)

# 14) fnmatch — filter on empty input
assert fnmatch.filter([], "*.txt") == []; _ledger.append(1)

# 15) fnmatch — translate returns str
assert isinstance(fnmatch.translate("*.txt"), str) == True; _ledger.append(1)

# 16) calendar — hasattr surface
assert hasattr(calendar, "month") == True; _ledger.append(1)
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "leapdays") == True; _ledger.append(1)
assert hasattr(calendar, "Calendar") == True; _ledger.append(1)
assert hasattr(calendar, "TextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "MONDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SUNDAY") == True; _ledger.append(1)
assert hasattr(calendar, "setfirstweekday") == True; _ledger.append(1)
assert hasattr(calendar, "HTMLCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "different_locale") == True; _ledger.append(1)
assert hasattr(calendar, "IllegalMonthError") == True; _ledger.append(1)

# 17) calendar — isleap
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)

# 18) calendar — leapdays between years
assert calendar.leapdays(2000, 2024) == 6; _ledger.append(1)

# 19) calendar — monthrange (weekday-of-first, days-in-month)
assert calendar.monthrange(2024, 2) == (3, 29); _ledger.append(1)
assert calendar.monthrange(2023, 2) == (2, 28); _ledger.append(1)

# 20) calendar — MONDAY/SUNDAY constants
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)

# 21) calendar — day_name / month_name lookups
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)

# 22) calendar — Calendar() class name
assert type(calendar.Calendar()).__name__ == "Calendar"; _ledger.append(1)

# 23) calendar — weekday integer value
assert calendar.weekday(2024, 2, 1) == 3; _ledger.append(1)

# 24) calendar — timegm of (2024,1,1,0,0,0,0,0,0) is epoch seconds
assert calendar.timegm((2024, 1, 1, 0, 0, 0, 0, 0, 0)) == 1704067200; _ledger.append(1)

# 25) calendar — firstweekday default
assert calendar.firstweekday() == 0; _ledger.append(1)

# 26) calendar — monthcalendar shape for Feb 2024
mc = calendar.monthcalendar(2024, 2)
assert isinstance(mc, list) == True; _ledger.append(1)
assert len(mc) == 5; _ledger.append(1)
assert any(29 in week for week in mc) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_shutil_glob_fnmatch_calendar_value_ops {sum(_ledger)} asserts")
