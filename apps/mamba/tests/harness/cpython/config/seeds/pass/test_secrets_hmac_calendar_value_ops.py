# Operational AssertionPass seed for the value-and-byte-shape surface
# of `secrets` (CSPRNG entry points used by every session-token /
# nonce / password-reset helper), `hmac` (the keyed-MAC primitive
# under JWT signing / cookie auth / webhook verification), and
# `calendar` (the weekday/month enum + UTC timegm helper). No fixture
# coverage yet for secrets or calendar; hmac has cross-fixture
# coverage but no isolated value pin.
#
# The matching subset between mamba and CPython is the value contract:
# token_bytes/token_hex return bytes/str of the requested length,
# randbelow stays in [0, N), randbits stays in [0, 2**k), secrets.
# choice returns one of the input items, compare_digest is True/False,
# hmac.new(...).hexdigest() matches the documented test vector for
# (b'k', b'msg', 'sha256'), hmac.digest() returns 32 bytes, every
# calendar.{MONDAY..SUNDAY} / {JANUARY..DECEMBER} sentinel has the
# documented integer value, calendar.day_name/day_abbr/month_name/
# month_abbr are length 7/7/13/13 with the documented English strings,
# calendar.isleap honours the 4/100/400 rule, calendar.leapdays counts
# the inclusive lower / exclusive upper window, and timegm reproduces
# the documented UTC epoch for 2024-01-01.
#
# Surface in this fixture:
#   • secrets.token_bytes(N) — returns `bytes` of length N;
#   • secrets.token_hex(N) — returns `str` of length 2*N;
#   • secrets.token_urlsafe(N) — returns `str`;
#   • secrets.choice([a,b,c]) — returns one of the input items;
#   • 0 <= secrets.randbelow(N) < N;
#   • 0 <= secrets.randbits(8) < 256;
#   • secrets.compare_digest('a','a') is True / ('a','b') is False;
#   • hasattr(secrets, 'SystemRandom') is True;
#   • hmac.new(b'k', b'msg', 'sha256').hexdigest() == documented vector
#     'bf1a0c1242929b6464a6c0a9ac6298a67e09bd1cd4ef1f182ce0141691fc17a0';
#   • hmac.compare_digest('a','a') True / ('a','b') False;
#   • hmac.digest(b'k', b'msg', 'sha256') — returns 32 bytes
#     (sha256 digest length);
#   • calendar.MONDAY..SUNDAY == 0..6 (weekday integer values);
#   • calendar.JANUARY..DECEMBER == 1..12 (month integer values);
#   • calendar.day_name[0..6] / day_abbr / month_name[1..12] /
#     month_abbr — documented English strings;
#   • len(day_name)==7, len(day_abbr)==7;
#   • len(month_name)==13, len(month_abbr)==13;
#   • calendar.isleap(2024) True, isleap(2023) False, isleap(2000) True
#     (every 400), isleap(1900) False (every 100 not 400);
#   • calendar.leapdays(2000, 2030) == 8;
#   • calendar.weekday(2024, 1, 1) == 0 (Monday);
#   • calendar.monthrange(2024, 2) — leap-year February yields 29 days
#     (second tuple element);
#   • calendar.monthrange(2023, 2) — non-leap February yields 28 days;
#   • calendar.timegm((2024, 1, 1, 0, 0, 0, 0, 0, 0)) == 1704067200
#     (documented UTC epoch).
#
# Behavioral edges that DIVERGE on mamba (calendar.{Day,Month} enum
# class identity, calendar.{Calendar,TextCalendar,HTMLCalendar} class
# identity, calendar.IllegalMonthError exception-class identity,
# zoneinfo.ZoneInfo class identity + instance construction, zoneinfo.
# available_timezones() returning a `set` containing 'UTC', secrets.
# SystemRandom() returning a SystemRandom instance with a `.random()`
# instance method) are covered in
# `lang_calendar_enum_zoneinfo_systemrandom_silent.py`.
import secrets
import hmac
import calendar

_ledger: list[int] = []

# 1) secrets.token_bytes — byte length contract
_b: bytes = secrets.token_bytes(8)
assert isinstance(_b, bytes); _ledger.append(1)
assert len(_b) == 8; _ledger.append(1)
_b16: bytes = secrets.token_bytes(16)
assert len(_b16) == 16; _ledger.append(1)
_b0: bytes = secrets.token_bytes(0)
assert len(_b0) == 0; _ledger.append(1)

# 2) secrets.token_hex — hex string of length 2*N
_h: str = secrets.token_hex(8)
assert isinstance(_h, str); _ledger.append(1)
assert len(_h) == 16; _ledger.append(1)
_h32: str = secrets.token_hex(32)
assert len(_h32) == 64; _ledger.append(1)

# 3) secrets.token_urlsafe — non-empty str
_u: str = secrets.token_urlsafe(8)
assert isinstance(_u, str); _ledger.append(1)
assert len(_u) > 0; _ledger.append(1)

# 4) secrets.choice — picks one of the given items
_choices = [10, 20, 30]
_picked = secrets.choice(_choices)
assert _picked in _choices; _ledger.append(1)

# 5) secrets.randbelow(N) — half-open [0, N)
_rb: int = secrets.randbelow(10)
assert isinstance(_rb, int); _ledger.append(1)
assert 0 <= _rb < 10; _ledger.append(1)
_rb_big: int = secrets.randbelow(1000000)
assert 0 <= _rb_big < 1000000; _ledger.append(1)

# 6) secrets.randbits(k) — half-open [0, 2**k)
_bits8: int = secrets.randbits(8)
assert isinstance(_bits8, int); _ledger.append(1)
assert 0 <= _bits8 < 256; _ledger.append(1)
_bits16: int = secrets.randbits(16)
assert 0 <= _bits16 < 65536; _ledger.append(1)

# 7) secrets.compare_digest — constant-time equality
assert secrets.compare_digest("alpha", "alpha") == True; _ledger.append(1)
assert secrets.compare_digest("alpha", "beta") == False; _ledger.append(1)

# 8) hasattr — SystemRandom is exposed
assert hasattr(secrets, "SystemRandom"); _ledger.append(1)

# 9) hmac.new — documented test vector for (b'k', b'msg', 'sha256')
_hmac_hex: str = hmac.new(b"k", b"msg", "sha256").hexdigest()
assert isinstance(_hmac_hex, str); _ledger.append(1)
assert _hmac_hex == "bf1a0c1242929b6464a6c0a9ac6298a67e09bd1cd4ef1f182ce0141691fc17a0"; _ledger.append(1)
assert len(_hmac_hex) == 64; _ledger.append(1)

# 10) hmac.compare_digest — constant-time equality on str and bytes
assert hmac.compare_digest("a", "a") == True; _ledger.append(1)
assert hmac.compare_digest("a", "b") == False; _ledger.append(1)
assert hmac.compare_digest(b"abc", b"abc") == True; _ledger.append(1)
assert hmac.compare_digest(b"abc", b"abd") == False; _ledger.append(1)

# 11) hmac.digest — one-shot sha256 digest, 32 bytes
_hd: bytes = hmac.digest(b"k", b"msg", "sha256")
assert isinstance(_hd, bytes); _ledger.append(1)
assert len(_hd) == 32; _ledger.append(1)

# 12) calendar weekday integer values
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.TUESDAY == 1; _ledger.append(1)
assert calendar.WEDNESDAY == 2; _ledger.append(1)
assert calendar.THURSDAY == 3; _ledger.append(1)
assert calendar.FRIDAY == 4; _ledger.append(1)
assert calendar.SATURDAY == 5; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)

# 13) calendar month integer values
assert calendar.JANUARY == 1; _ledger.append(1)
assert calendar.DECEMBER == 12; _ledger.append(1)

# 14) calendar.day_name / day_abbr — documented English strings
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.day_name[6] == "Sunday"; _ledger.append(1)
assert calendar.day_abbr[0] == "Mon"; _ledger.append(1)
assert calendar.day_abbr[6] == "Sun"; _ledger.append(1)
assert len(calendar.day_name) == 7; _ledger.append(1)
assert len(calendar.day_abbr) == 7; _ledger.append(1)

# 15) calendar.month_name / month_abbr — index 0 is empty, 1..12 are names
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)
assert calendar.month_abbr[1] == "Jan"; _ledger.append(1)
assert calendar.month_abbr[12] == "Dec"; _ledger.append(1)
assert len(calendar.month_name) == 13; _ledger.append(1)
assert len(calendar.month_abbr) == 13; _ledger.append(1)

# 16) calendar.isleap — 4/100/400 rule
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)

# 17) calendar.leapdays — count over a window
assert calendar.leapdays(2000, 2030) == 8; _ledger.append(1)

# 18) calendar.weekday — 2024-01-01 was a Monday
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)

# 19) calendar.monthrange — second tuple element is the day count
_mr_2024: tuple = calendar.monthrange(2024, 2)
assert isinstance(_mr_2024, tuple); _ledger.append(1)
assert len(_mr_2024) == 2; _ledger.append(1)
assert _mr_2024[1] == 29; _ledger.append(1)
_mr_2023: tuple = calendar.monthrange(2023, 2)
assert _mr_2023[1] == 28; _ledger.append(1)

# 20) calendar.timegm — UTC seconds-since-epoch for 2024-01-01 00:00 UTC
assert calendar.timegm((2024, 1, 1, 0, 0, 0, 0, 0, 0)) == 1704067200; _ledger.append(1)

# NB: calendar.Day / Month enum class identity, calendar.{Calendar,
# TextCalendar, HTMLCalendar} class identity, calendar.
# IllegalMonthError exception-class identity, zoneinfo.ZoneInfo
# class identity + instance construction, zoneinfo.available_timezones
# returning a `set` containing 'UTC', and secrets.SystemRandom()
# returning an instance with `.random()` all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_secrets_hmac_calendar_value_ops {sum(_ledger)} asserts")
