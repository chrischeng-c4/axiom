# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_cmath_difflib_configparser_silent"
# subject = "cpython321.lang_cmath_difflib_configparser_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_cmath_difflib_configparser_silent.py"
# status = "filled"
# ///
"""cpython321.lang_cmath_difflib_configparser_silent: execute CPython 3.12 seed lang_cmath_difflib_configparser_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `cmath.pi == 3.141592653589793` (the
# documented "cmath.pi is the float pi" — mamba returns
# 4614256656552045848 — i64 bit pattern of the double instead of the
# float), `cmath.tau == 6.283185307179586` (the documented "cmath.tau
# is the float 2*pi" — mamba returns 4618760256179416344 — i64 bit
# pattern of the double), `cmath.e == 2.718281828459045` (the
# documented "cmath.e is the float e" — mamba returns
# 4613303445314885481 — i64 bit pattern of the double), `hasattr(
# difflib, 'Differ')` (the documented "difflib exposes the Differ
# class" — mamba returns False), `hasattr(difflib, 'ndiff')` (the
# documented "difflib exposes the ndiff helper" — mamba returns
# False), `hasattr(difflib, 'context_diff')` (the documented "difflib
# exposes the context_diff helper" — mamba returns False), `hasattr(
# configparser, 'RawConfigParser')` (the documented "configparser
# exposes the RawConfigParser class" — mamba returns False), `
# configparser.DEFAULTSECT == 'DEFAULT'` (the documented "configparser
# .DEFAULTSECT is the constant 'DEFAULT'" — mamba returns None —
# constant unresolved), `type(configparser.ConfigParser()).__name__ ==
# 'ConfigParser'` (the documented "configparser.ConfigParser()
# constructs a ConfigParser instance" — mamba returns 'dict' —
# constructor degrades to plain dict), and `hasattr(csv, 'Sniffer')`
# (the documented "csv exposes the Sniffer dialect detector" — mamba
# returns False).
# Ten-pack pinned to atomic 307.
#
# Behavioral edges that CONFORM on mamba (cmath — hasattr sqrt/exp/
# log/cos/sin/pi/e/tau/isfinite/isinf/isnan/polar/rect/phase + cmath.
# sqrt(-1) == 1j + cmath.isnan(1+1j) False. difflib — hasattr Sequence
# Matcher/unified_diff/get_close_matches + get_close_matches ranks
# 'hellp' first. shlex — hasattr split/join/quote + shlex.split tokens
# + shlex.quote escapes spaces + shlex.join round-trip. configparser —
# hasattr ConfigParser only. csv — hasattr reader/writer/DictReader/
# DictWriter/Dialect/excel/excel_tab/unix_dialect/QUOTE_ALL/QUOTE_
# MINIMAL/QUOTE_NONE/QUOTE_NONNUMERIC/Error/field_size_limit. secrets
# — hasattr token_bytes/token_hex/token_urlsafe/choice/randbelow/
# randbits/compare_digest/SystemRandom + len(token_bytes(16)) == 16 +
# len(token_hex(8)) == 16 + compare_digest equal/unequal) are covered
# in the matching pass fixture
# `test_cmath_difflib_shlex_csv_secrets_value_ops`.
import cmath
import difflib
import configparser
import csv


_ledger: list[int] = []

# 1) cmath.pi == 3.141592653589793 — float pi
#    (mamba: returns 4614256656552045848 — i64 bit pattern of the double)
assert cmath.pi == 3.141592653589793; _ledger.append(1)

# 2) cmath.tau == 6.283185307179586 — float 2*pi
#    (mamba: returns 4618760256179416344 — i64 bit pattern of the double)
assert cmath.tau == 6.283185307179586; _ledger.append(1)

# 3) cmath.e == 2.718281828459045 — float e
#    (mamba: returns 4613303445314885481 — i64 bit pattern of the double)
assert cmath.e == 2.718281828459045; _ledger.append(1)

# 4) hasattr(difflib, 'Differ') — Differ class
#    (mamba: returns False)
assert hasattr(difflib, "Differ") == True; _ledger.append(1)

# 5) hasattr(difflib, 'ndiff') — ndiff helper
#    (mamba: returns False)
assert hasattr(difflib, "ndiff") == True; _ledger.append(1)

# 6) hasattr(difflib, 'context_diff') — context_diff helper
#    (mamba: returns False)
assert hasattr(difflib, "context_diff") == True; _ledger.append(1)

# 7) hasattr(configparser, 'RawConfigParser') — RawConfigParser class
#    (mamba: returns False)
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)

# 8) configparser.DEFAULTSECT == 'DEFAULT' — default-section constant
#    (mamba: returns None — constant unresolved)
assert configparser.DEFAULTSECT == "DEFAULT"; _ledger.append(1)

# 9) type(configparser.ConfigParser()).__name__ == 'ConfigParser' — ConfigParser instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(configparser.ConfigParser()).__name__ == "ConfigParser"; _ledger.append(1)

# 10) hasattr(csv, 'Sniffer') — Sniffer dialect detector
#     (mamba: returns False)
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_cmath_difflib_configparser_silent {sum(_ledger)} asserts")
