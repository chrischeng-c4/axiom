# Atomic 276 pass conformance — pathlib module (hasattr Path/PurePath/
# PurePosixPath/PureWindowsPath/PosixPath/WindowsPath) + zipfile
# module (hasattr ZipFile/ZIP_STORED/ZIP_DEFLATED/is_zipfile +
# ZIP_STORED==0 / ZIP_DEFLATED==8) + tarfile module (hasattr open/
# is_tarfile) + calendar module (hasattr Calendar/TextCalendar/HTML
# Calendar/monthrange/isleap/weekday/month_name/day_name/month_abbr/
# day_abbr/MONDAY/TUESDAY/SUNDAY/calendar/month/timegm/leapdays +
# isleap 2020/2000 True / 2021/1900 False + monthrange 2020/2021
# Feb + weekday 2020-01-01 == 2 + MONDAY==0 / SUNDAY==6 + leapdays
# 2000..2020).
# All asserts match between CPython 3.12 and mamba.
import pathlib
import zipfile
import tarfile
import calendar


_ledger: list[int] = []

# 1) pathlib — hasattr class surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "WindowsPath") == True; _ledger.append(1)

# 2) zipfile — hasattr core + constants
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)

# 3) zipfile — compression-method constants
assert zipfile.ZIP_STORED == 0; _ledger.append(1)
assert zipfile.ZIP_DEFLATED == 8; _ledger.append(1)

# 4) tarfile — hasattr minimal surface
assert hasattr(tarfile, "open") == True; _ledger.append(1)
assert hasattr(tarfile, "is_tarfile") == True; _ledger.append(1)

# 5) calendar — hasattr class surface
assert hasattr(calendar, "Calendar") == True; _ledger.append(1)
assert hasattr(calendar, "TextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "HTMLCalendar") == True; _ledger.append(1)

# 6) calendar — hasattr helper surface
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "weekday") == True; _ledger.append(1)
assert hasattr(calendar, "calendar") == True; _ledger.append(1)
assert hasattr(calendar, "month") == True; _ledger.append(1)
assert hasattr(calendar, "timegm") == True; _ledger.append(1)
assert hasattr(calendar, "leapdays") == True; _ledger.append(1)

# 7) calendar — hasattr label tables + weekday constants
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "month_abbr") == True; _ledger.append(1)
assert hasattr(calendar, "day_abbr") == True; _ledger.append(1)
assert hasattr(calendar, "MONDAY") == True; _ledger.append(1)
assert hasattr(calendar, "TUESDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SUNDAY") == True; _ledger.append(1)

# 8) calendar — isleap value contracts
assert calendar.isleap(2020) == True; _ledger.append(1)
assert calendar.isleap(2021) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)

# 9) calendar — monthrange/weekday value contracts
assert calendar.monthrange(2020, 2) == (5, 29); _ledger.append(1)
assert calendar.monthrange(2021, 2) == (0, 28); _ledger.append(1)
assert calendar.weekday(2020, 1, 1) == 2; _ledger.append(1)

# 10) calendar — weekday constants + leapdays
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)
assert calendar.leapdays(2000, 2020) == 5; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pathlib_zipfile_tarfile_calendar_value_ops {sum(_ledger)} asserts")
