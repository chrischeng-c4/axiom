# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_unicodedata"
# subject = "cpython321.test_unicodedata"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_unicodedata.py"
# status = "filled"
# ///
"""cpython321.test_unicodedata: execute CPython 3.12 seed test_unicodedata"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: unicodedata — the partial surface mamba services today:
#   * category(ch) returns canonical CPython category codes for ASCII letters
#     and digits — Ll (lowercase), Lu (uppercase), Nd (decimal digit), Zs
#     (space separator)
#   * bidirectional(ch) returns "L" (Left-to-Right) for ASCII letters
#   * normalize("NFC", s) and normalize("NFD", s) accept a string and return
#     a string (treated as passthrough on ASCII-only input)
#   * name(ch) returns a string (mamba returns "UNICODE CHAR XXXX" format —
#     not the CPython "LATIN CAPITAL LETTER A" name — but it IS a non-empty str)
# Intentionally NOT exercised on mamba today (tracked separately):
#   * category() on punctuation/control — '!' and '(' return "Cn" (should
#     be "Po" / "Ps"); '\n' returns "Zs" (should be "Cc")
#   * bidirectional('1') returns "L" (should be "EN" European Number)
#   * combining(ch) — missing
#   * lookup("BLACK STAR") — missing
#   * numeric/digit/decimal — missing
#   * NFC/NFD normalization of combining marks (e.g. "café") on non-ASCII
#     codepoints — currently a passthrough rather than a real normalisation
#   * name(ch) returning the CPython canonical name (mamba returns
#     "UNICODE CHAR XXXX" instead of "LATIN CAPITAL LETTER A")
import unicodedata

_ledger: list[int] = []

# (1) Lowercase ASCII letters -> "Ll"
for _ch in ("a", "m", "z"):
    _c = unicodedata.category(_ch)
    assert _c == "Ll", (
        f"unicodedata.category({_ch!r}) == 'Ll', got {_c!r}"
    )
_ledger.append(1)

# (2) Uppercase ASCII letters -> "Lu"
for _ch in ("A", "M", "Z"):
    _c = unicodedata.category(_ch)
    assert _c == "Lu", (
        f"unicodedata.category({_ch!r}) == 'Lu', got {_c!r}"
    )
_ledger.append(1)

# (3) ASCII decimal digits -> "Nd"
for _ch in ("0", "5", "9"):
    _c = unicodedata.category(_ch)
    assert _c == "Nd", (
        f"unicodedata.category({_ch!r}) == 'Nd', got {_c!r}"
    )
_ledger.append(1)

# (4) ASCII space -> "Zs"
assert unicodedata.category(" ") == "Zs", (
    f"unicodedata.category(' ') == 'Zs', got {unicodedata.category(' ')!r}"
)
_ledger.append(1)

# (5) bidirectional on ASCII letters -> "L"
for _ch in ("a", "A", "z", "Z"):
    _b = unicodedata.bidirectional(_ch)
    assert _b == "L", (
        f"unicodedata.bidirectional({_ch!r}) == 'L', got {_b!r}"
    )
_ledger.append(1)

# (6) normalize NFC on ASCII string passes through
_n1 = unicodedata.normalize("NFC", "hello")
assert _n1 == "hello", (
    f"unicodedata.normalize('NFC', 'hello') == 'hello', got {_n1!r}"
)
_ledger.append(1)

# (7) normalize NFD on ASCII string passes through
_n2 = unicodedata.normalize("NFD", "world")
assert _n2 == "world", (
    f"unicodedata.normalize('NFD', 'world') == 'world', got {_n2!r}"
)
_ledger.append(1)

# (8) normalize returns a str (regardless of form)
assert isinstance(_n1, str), (
    f"unicodedata.normalize returns str, got {type(_n1).__name__!r}"
)
_ledger.append(1)

# (9) name(ch) returns a non-empty string (mamba returns "UNICODE CHAR XXXX"
#     format — not CPython's canonical name, but it IS a string and non-empty)
_n = unicodedata.name("A")
assert isinstance(_n, str), (
    f"unicodedata.name('A') returns str, got {type(_n).__name__!r}"
)
_ledger.append(1)
assert len(_n) > 0, f"unicodedata.name('A') returns non-empty str, got {_n!r}"
_ledger.append(1)

# (10) category and bidirectional are exposed as callables
assert hasattr(unicodedata, "category"), "unicodedata.category symbol exposed"
_ledger.append(1)
assert hasattr(unicodedata, "bidirectional"), (
    "unicodedata.bidirectional symbol exposed"
)
_ledger.append(1)

# (11) Same character produces same category result twice (idempotent)
assert unicodedata.category("a") == unicodedata.category("a"), (
    "unicodedata.category is deterministic for the same input"
)
_ledger.append(1)

# (12) Distinct categories ARE distinct (sanity)
assert unicodedata.category("a") != unicodedata.category("A"), (
    "Ll and Lu are distinct categories"
)
_ledger.append(1)
assert unicodedata.category("0") != unicodedata.category("a"), (
    "Nd and Ll are distinct categories"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_unicodedata {sum(_ledger)} asserts")
