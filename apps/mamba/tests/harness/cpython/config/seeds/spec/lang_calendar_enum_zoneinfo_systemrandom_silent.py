# Operational AssertionPass seed for SILENT divergences in `calendar`
# (Day/Month enum class identity, Calendar/TextCalendar/HTMLCalendar
# class identity, IllegalMonthError exception-class identity),
# `zoneinfo` (ZoneInfo class identity, ZoneInfo('UTC') instance
# construction, available_timezones returning a `set` containing
# 'UTC', ZoneInfoNotFoundError exception-class identity), and
# `secrets` (SystemRandom class identity, SystemRandom().random()
# instance method).
#
# The matching subset (secrets token/randbelow/randbits/compare_digest
# byte-and-length contract, hmac.new/digest/compare_digest value
# vectors, calendar weekday/month integer sentinels, calendar isleap/
# leapdays/weekday/monthrange/timegm value contract, calendar.day_name
# / month_name English strings) is covered by
# `test_secrets_hmac_calendar_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(calendar.MONDAY).__name__ == "Day" — Day IntEnum identity
#     (mamba: returns "int", raw integer sentinel);
#   • type(calendar.JANUARY).__name__ == "Month" — Month IntEnum
#     identity (mamba: returns "int");
#   • calendar.Calendar.__name__ == "Calendar" — class identity
#     (mamba: returns None);
#   • calendar.TextCalendar.__name__ == "TextCalendar"
#     (mamba: returns None);
#   • calendar.HTMLCalendar.__name__ == "HTMLCalendar"
#     (mamba: returns None);
#   • calendar.Calendar() — returns Calendar instance with
#     `.iterweekdays()` and `.iterweekdates(year, month)` methods
#     (mamba: returns a stub instance with no methods);
#   • calendar.IllegalMonthError — class (type), not an instance
#     (mamba: returns a stub `IllegalMonthError` instance);
#   • zoneinfo.ZoneInfo.__name__ == "ZoneInfo" — class identity
#     (mamba: returns None);
#   • zoneinfo.ZoneInfo("UTC") — returns ZoneInfo instance with
#     `.key == "UTC"` (mamba: returns plain dict);
#   • type(zoneinfo.available_timezones()).__name__ == "set" — return
#     type contract (mamba: returns a list);
#   • "UTC" in zoneinfo.available_timezones() — UTC is always present
#     (mamba: returns False — available_timezones list is empty);
#   • zoneinfo.ZoneInfoNotFoundError — exception class (type)
#     (mamba: returns a function lambda);
#   • secrets.SystemRandom.__name__ == "SystemRandom" — class identity
#     (mamba: returns None);
#   • secrets.SystemRandom() — returns SystemRandom instance with
#     `.random()` method (mamba: returns int);
#   • secrets.SystemRandom().random() — float in [0.0, 1.0)
#     (mamba: AttributeError, instance is int not random.SystemRandom).
import calendar as _calendar_mod
import zoneinfo as _zi_mod
import secrets as _secrets_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing on attribute access — `calendar.Calendar` /
# `calendar.IllegalMonthError`, `zoneinfo.ZoneInfo` /
# `zoneinfo.ZoneInfoNotFoundError`, and `secrets.SystemRandom` are
# documented public class objects that mamba elides at the type-stub
# level.
calendar: Any = _calendar_mod
zoneinfo: Any = _zi_mod
secrets: Any = _secrets_mod

_ledger: list[int] = []

# 1) calendar Day/Month enum class identity
assert type(calendar.MONDAY).__name__ == "Day"; _ledger.append(1)
assert type(calendar.SUNDAY).__name__ == "Day"; _ledger.append(1)
assert type(calendar.JANUARY).__name__ == "Month"; _ledger.append(1)
assert type(calendar.DECEMBER).__name__ == "Month"; _ledger.append(1)

# 2) calendar.Calendar class identity + instance method surface
assert calendar.Calendar.__name__ == "Calendar"; _ledger.append(1)
_cal: Any = calendar.Calendar()
assert type(_cal).__name__ == "Calendar"; _ledger.append(1)
# Calendar instance has documented iter helpers
_wd: Any = list(_cal.iterweekdays())
assert _wd == [0, 1, 2, 3, 4, 5, 6]; _ledger.append(1)

# 3) calendar.TextCalendar / HTMLCalendar class identity
assert calendar.TextCalendar.__name__ == "TextCalendar"; _ledger.append(1)
assert calendar.HTMLCalendar.__name__ == "HTMLCalendar"; _ledger.append(1)

# 4) calendar.IllegalMonthError — exception class (type), not instance
assert type(calendar.IllegalMonthError).__name__ == "type"; _ledger.append(1)

# 5) zoneinfo.ZoneInfo — class identity
assert zoneinfo.ZoneInfo.__name__ == "ZoneInfo"; _ledger.append(1)

# 6) ZoneInfo("UTC") — instance with .key attribute
_zone: Any = zoneinfo.ZoneInfo("UTC")
assert type(_zone).__name__ == "ZoneInfo"; _ledger.append(1)
assert _zone.key == "UTC"; _ledger.append(1)

# 7) available_timezones returns a `set` containing 'UTC'
_tz: Any = zoneinfo.available_timezones()
assert type(_tz).__name__ == "set"; _ledger.append(1)
assert "UTC" in _tz; _ledger.append(1)

# 8) ZoneInfoNotFoundError is an exception class (type), not a lambda
assert type(zoneinfo.ZoneInfoNotFoundError).__name__ == "type"; _ledger.append(1)

# 9) secrets.SystemRandom — class identity
assert secrets.SystemRandom.__name__ == "SystemRandom"; _ledger.append(1)

# 10) SystemRandom() — instance with .random() method
_sr: Any = secrets.SystemRandom()
assert type(_sr).__name__ == "SystemRandom"; _ledger.append(1)
_r: Any = _sr.random()
assert isinstance(_r, float); _ledger.append(1)
assert 0.0 <= _r < 1.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_calendar_enum_zoneinfo_systemrandom_silent {sum(_ledger)} asserts")
