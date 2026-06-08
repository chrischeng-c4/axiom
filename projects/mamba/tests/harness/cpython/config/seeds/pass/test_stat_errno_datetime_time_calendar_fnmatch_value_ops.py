# Atomic 226 pass conformance — stat/errno/datetime/time/calendar/fnmatch/
# sys/zoneinfo value ops that match between CPython 3.12 and mamba.
import stat
import errno
import datetime
import time
import calendar
import fnmatch
import sys
import zoneinfo

_ledger: list[int] = []

# 1) stat — permission masks
assert stat.S_IRWXU == 0o700; _ledger.append(1)
assert stat.S_IRUSR == 0o400; _ledger.append(1)
assert stat.S_IWUSR == 0o200; _ledger.append(1)
assert stat.S_IXUSR == 0o100; _ledger.append(1)
assert stat.S_IRGRP == 0o40; _ledger.append(1)
assert stat.S_IWGRP == 0o20; _ledger.append(1)
assert stat.S_IXGRP == 0o10; _ledger.append(1)
assert stat.S_IROTH == 0o4; _ledger.append(1)
assert stat.S_IWOTH == 0o2; _ledger.append(1)
assert stat.S_IXOTH == 0o1; _ledger.append(1)
assert stat.S_IRWXG == 0o70; _ledger.append(1)
assert stat.S_IRWXO == 0o7; _ledger.append(1)

# 2) stat — file-type predicates
assert stat.S_ISDIR(0o040755) == True; _ledger.append(1)
assert stat.S_ISREG(0o100644) == True; _ledger.append(1)
assert stat.S_ISLNK(0o120755) == True; _ledger.append(1)
assert stat.S_ISDIR(0o100644) == False; _ledger.append(1)
assert stat.S_ISREG(0o040755) == False; _ledger.append(1)
assert stat.S_IMODE(0o100644) == 0o644; _ledger.append(1)

# 3) stat — ST_* indices
assert stat.ST_MODE == 0; _ledger.append(1)
assert stat.ST_INO == 1; _ledger.append(1)
assert stat.ST_DEV == 2; _ledger.append(1)
assert stat.ST_NLINK == 3; _ledger.append(1)
assert stat.ST_UID == 4; _ledger.append(1)
assert stat.ST_GID == 5; _ledger.append(1)
assert stat.ST_SIZE == 6; _ledger.append(1)
assert stat.ST_ATIME == 7; _ledger.append(1)
assert stat.ST_MTIME == 8; _ledger.append(1)
assert stat.ST_CTIME == 9; _ledger.append(1)

# 4) errno — canonical POSIX codes are positive integers
assert errno.EPERM > 0; _ledger.append(1)
assert errno.ENOENT > 0; _ledger.append(1)
assert errno.EIO > 0; _ledger.append(1)
assert errno.EBADF > 0; _ledger.append(1)
assert errno.EACCES > 0; _ledger.append(1)
assert errno.EEXIST > 0; _ledger.append(1)
assert errno.ENOTDIR > 0; _ledger.append(1)
assert errno.EISDIR > 0; _ledger.append(1)
assert errno.EINVAL > 0; _ledger.append(1)
assert errno.ENOSPC > 0; _ledger.append(1)
assert errno.EPIPE > 0; _ledger.append(1)

# 5) datetime — basic constructor accessors
_d = datetime.date(2024, 1, 15)
assert _d.year == 2024; _ledger.append(1)
assert _d.month == 1; _ledger.append(1)
assert _d.day == 15; _ledger.append(1)

_dt = datetime.datetime(2024, 6, 7, 12, 30, 45)
assert _dt.year == 2024; _ledger.append(1)
assert _dt.month == 6; _ledger.append(1)
assert _dt.day == 7; _ledger.append(1)
assert _dt.hour == 12; _ledger.append(1)
assert _dt.minute == 30; _ledger.append(1)
assert _dt.second == 45; _ledger.append(1)

_td = datetime.timedelta(days=2, seconds=3600)
assert _td.days == 2; _ledger.append(1)
assert _td.seconds == 3600; _ledger.append(1)

# 6) time — gmtime(0) is the Unix epoch
_gm = time.gmtime(0)
assert _gm.tm_year == 1970; _ledger.append(1)
assert _gm.tm_mon == 1; _ledger.append(1)
assert _gm.tm_mday == 1; _ledger.append(1)
assert _gm.tm_hour == 0; _ledger.append(1)
assert _gm.tm_min == 0; _ledger.append(1)
assert _gm.tm_sec == 0; _ledger.append(1)
assert time.strftime("%Y-%m-%d", _gm) == "1970-01-01"; _ledger.append(1)

# 7) calendar — value contracts
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)
assert calendar.leapdays(2000, 2024) == 6; _ledger.append(1)
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.TUESDAY == 1; _ledger.append(1)
assert calendar.WEDNESDAY == 2; _ledger.append(1)
assert calendar.THURSDAY == 3; _ledger.append(1)
assert calendar.FRIDAY == 4; _ledger.append(1)
assert calendar.SATURDAY == 5; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)
_mr = calendar.monthrange(2024, 2)
assert _mr[1] == 29; _ledger.append(1)
_mr2 = calendar.monthrange(2023, 2)
assert _mr2[1] == 28; _ledger.append(1)

# 8) fnmatch
assert fnmatch.fnmatch("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("Foo.PY", "*.py") == False; _ledger.append(1)
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)

# 9) sys — invariants
assert sys.byteorder in ("little", "big"); _ledger.append(1)
assert sys.version_info.major == 3; _ledger.append(1)
assert sys.maxsize > 0; _ledger.append(1)
assert isinstance(sys.platform, str); _ledger.append(1)
assert len(sys.argv) >= 0; _ledger.append(1)

# 10) zoneinfo — surface attrs
assert hasattr(zoneinfo, "ZoneInfo") == True; _ledger.append(1)
assert hasattr(zoneinfo, "available_timezones") == True; _ledger.append(1)
assert hasattr(zoneinfo, "TZPATH") == True; _ledger.append(1)
assert hasattr(zoneinfo, "ZoneInfoNotFoundError") == True; _ledger.append(1)
assert hasattr(zoneinfo, "InvalidTZPathWarning") == True; _ledger.append(1)
assert hasattr(zoneinfo, "reset_tzpath") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_stat_errno_datetime_time_calendar_fnmatch_value_ops {sum(_ledger)} asserts")
