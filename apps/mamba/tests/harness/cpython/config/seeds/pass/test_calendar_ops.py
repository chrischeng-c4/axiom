# Operational AssertionPass seed for the `calendar` stdlib module.
# Surface: isleap, monthrange (first-weekday, days-in-month),
# weekday (0=Mon..6=Sun).
# Companion to stub/test_calendar.py — vendored unittest seed.
import calendar
_ledger: list[int] = []
assert calendar.isleap(2024); _ledger.append(1)
assert not calendar.isleap(2023); _ledger.append(1)
assert not calendar.isleap(1900); _ledger.append(1)
assert calendar.isleap(2000); _ledger.append(1)
assert calendar.monthrange(2024, 2) == (3, 29); _ledger.append(1)
assert calendar.monthrange(2023, 2) == (2, 28); _ledger.append(1)
assert calendar.monthrange(2024, 1)[1] == 31; _ledger.append(1)
assert calendar.monthrange(2024, 4)[1] == 30; _ledger.append(1)
assert calendar.weekday(2024, 3, 15) == 4; _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_calendar_ops {sum(_ledger)} asserts")
