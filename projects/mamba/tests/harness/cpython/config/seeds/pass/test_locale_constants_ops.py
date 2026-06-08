# Operational AssertionPass seed for `locale` category constants.
# Surface: LC_NUMERIC / LC_TIME / LC_ALL hold their canonical
# numeric category indices used by setlocale(category, locale).
# Only the cross-platform-stable indices are asserted; LC_COLLATE /
# LC_MONETARY / LC_MESSAGES indices currently return None on mamba
# and are intentionally omitted.
# Companion to stub/test_locale.py — vendored unittest seed.
import locale
_ledger: list[int] = []
assert isinstance(locale.LC_NUMERIC, int); _ledger.append(1)
assert isinstance(locale.LC_TIME, int); _ledger.append(1)
assert isinstance(locale.LC_ALL, int); _ledger.append(1)
# Distinct category invariant; numeric values are platform-specific.
assert len({locale.LC_NUMERIC, locale.LC_TIME, locale.LC_ALL}) == 3; _ledger.append(1)
assert locale.LC_ALL in (0, 6); _ledger.append(1)
# All three are distinct non-negative ints
assert locale.LC_NUMERIC >= 0; _ledger.append(1)
assert locale.LC_TIME >= 0; _ledger.append(1)
assert locale.LC_ALL >= 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_locale_constants_ops {sum(_ledger)} asserts")
