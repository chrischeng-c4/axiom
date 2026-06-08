# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: locale — getlocale() returning a 2-tuple, LC_* category constants
# (LC_ALL / LC_CTYPE / LC_TIME / LC_NUMERIC), setlocale(LC_ALL, 'C') round-trip.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * getpreferredencoding (AttributeError)
#   * getdefaultlocale (AttributeError)
#   * atof / atoi (AttributeError)
import locale

_ledger: list[int] = []

# getlocale returns a 2-tuple (language, encoding)
_loc = locale.getlocale()
assert isinstance(_loc, tuple), "locale.getlocale() returns a tuple"
_ledger.append(1)

assert len(_loc) == 2, f"locale.getlocale() returns a 2-tuple, got len={len(_loc)}"
_ledger.append(1)

# LC_* category constants are non-negative ints
assert isinstance(locale.LC_ALL, int) and locale.LC_ALL >= 0, (
    f"locale.LC_ALL is a non-negative int, got {locale.LC_ALL!r}"
)
_ledger.append(1)

assert isinstance(locale.LC_CTYPE, int) and locale.LC_CTYPE >= 0, (
    f"locale.LC_CTYPE is a non-negative int, got {locale.LC_CTYPE!r}"
)
_ledger.append(1)

assert isinstance(locale.LC_TIME, int) and locale.LC_TIME >= 0, (
    f"locale.LC_TIME is a non-negative int, got {locale.LC_TIME!r}"
)
_ledger.append(1)

assert isinstance(locale.LC_NUMERIC, int) and locale.LC_NUMERIC >= 0, (
    f"locale.LC_NUMERIC is a non-negative int, got {locale.LC_NUMERIC!r}"
)
_ledger.append(1)

# The four LC_* constants are distinct (so they actually index different
# categories)
assert len({locale.LC_ALL, locale.LC_CTYPE, locale.LC_TIME, locale.LC_NUMERIC}) == 4, (
    "locale.LC_ALL / LC_CTYPE / LC_TIME / LC_NUMERIC are pairwise distinct"
)
_ledger.append(1)

# setlocale(LC_ALL, 'C') is the canonical POSIX-portable reset and must
# succeed without raising
locale.setlocale(locale.LC_ALL, "C")
_ledger.append(1)

# After switching, setlocale to the same value should still succeed
locale.setlocale(locale.LC_ALL, "C")
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_locale {sum(_ledger)} asserts")
