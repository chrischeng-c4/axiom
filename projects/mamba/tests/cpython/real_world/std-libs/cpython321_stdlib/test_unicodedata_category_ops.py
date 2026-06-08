# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_unicodedata_category_ops"
# subject = "cpython321.test_unicodedata_category_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_unicodedata_category_ops.py"
# status = "filled"
# ///
"""cpython321.test_unicodedata_category_ops: execute CPython 3.12 seed test_unicodedata_category_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the unicodedata-module classification
# and normalization surface. Surface: unicodedata.category(c) returns the
# canonical two-letter Unicode general category for letters (Lu/Ll), digits
# (Nd), and the ASCII space (Zs); category honors latin-1 supplement
# lowercase (`é` → Ll); unicodedata.decimal(c) returns the integer value of
# an ASCII decimal digit '0'..'9'; unicodedata.bidirectional(c) returns the
# Bidi class for an ASCII letter ('A' → 'L'); unicodedata.normalize(form,
# str) is identity for plain-ASCII input across all four canonical/
# compatibility forms (NFC, NFD, NFKC, NFKD). Companion to test_string_*
# (which cover the string-method surface).
import unicodedata
_ledger: list[int] = []

# category — ASCII letters Lu / Ll
assert unicodedata.category("A") == "Lu"; _ledger.append(1)
assert unicodedata.category("a") == "Ll"; _ledger.append(1)
assert unicodedata.category("Z") == "Lu"; _ledger.append(1)
assert unicodedata.category("z") == "Ll"; _ledger.append(1)

# category — ASCII digits Nd
assert unicodedata.category("0") == "Nd"; _ledger.append(1)
assert unicodedata.category("1") == "Nd"; _ledger.append(1)
assert unicodedata.category("9") == "Nd"; _ledger.append(1)

# category — ASCII space Zs
assert unicodedata.category(" ") == "Zs"; _ledger.append(1)

# category — latin-1 lowercase letter with diacritic
assert unicodedata.category("é") == "Ll"; _ledger.append(1)

# decimal — integer value of ASCII digit
assert unicodedata.decimal("0") == 0; _ledger.append(1)
assert unicodedata.decimal("1") == 1; _ledger.append(1)
assert unicodedata.decimal("2") == 2; _ledger.append(1)
assert unicodedata.decimal("5") == 5; _ledger.append(1)
assert unicodedata.decimal("9") == 9; _ledger.append(1)

# bidirectional — left-to-right ASCII letter
assert unicodedata.bidirectional("A") == "L"; _ledger.append(1)

# normalize — identity for plain ASCII across all four canonical forms
assert unicodedata.normalize("NFC", "abc") == "abc"; _ledger.append(1)
assert unicodedata.normalize("NFD", "abc") == "abc"; _ledger.append(1)
assert unicodedata.normalize("NFKC", "abc") == "abc"; _ledger.append(1)
assert unicodedata.normalize("NFKD", "abc") == "abc"; _ledger.append(1)
assert unicodedata.normalize("NFC", "hello") == "hello"; _ledger.append(1)
assert unicodedata.normalize("NFD", "world") == "world"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_unicodedata_category_ops {sum(_ledger)} asserts")
