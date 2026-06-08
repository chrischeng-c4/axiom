# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_cmath_difflib_shlex_csv_secrets_value_ops"
# subject = "cpython321.test_cmath_difflib_shlex_csv_secrets_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_cmath_difflib_shlex_csv_secrets_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_cmath_difflib_shlex_csv_secrets_value_ops: execute CPython 3.12 seed test_cmath_difflib_shlex_csv_secrets_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 307 pass conformance — cmath module (hasattr sqrt/exp/log/cos
# /sin/pi/e/tau/isfinite/isinf/isnan/polar/rect/phase + cmath.sqrt(-1)
# == 1j + cmath.isnan(1+1j) False) + difflib module (hasattr Sequence
# Matcher/unified_diff/get_close_matches + get_close_matches ranks
# 'hellp' first) + shlex module (hasattr split/join/quote + shlex.
# split 'a b c' tokens + shlex.quote escapes spaces + shlex.join round-
# trip) + configparser module (hasattr ConfigParser) + csv module
# (hasattr reader/writer/DictReader/DictWriter/Dialect/excel/excel_tab/
# unix_dialect/QUOTE_ALL/QUOTE_MINIMAL/QUOTE_NONE/QUOTE_NONNUMERIC/
# Error/field_size_limit) + secrets module (hasattr token_bytes/token_
# hex/token_urlsafe/choice/randbelow/randbits/compare_digest/System
# Random + len(token_bytes(16)) == 16 + len(token_hex(8)) == 16 +
# compare_digest equal/unequal).
# All asserts match between CPython 3.12 and mamba.
import cmath
import difflib
import shlex
import configparser
import csv
import secrets


_ledger: list[int] = []

# 1) cmath — hasattr core surface
assert hasattr(cmath, "sqrt") == True; _ledger.append(1)
assert hasattr(cmath, "exp") == True; _ledger.append(1)
assert hasattr(cmath, "log") == True; _ledger.append(1)
assert hasattr(cmath, "cos") == True; _ledger.append(1)
assert hasattr(cmath, "sin") == True; _ledger.append(1)
assert hasattr(cmath, "pi") == True; _ledger.append(1)
assert hasattr(cmath, "e") == True; _ledger.append(1)
assert hasattr(cmath, "tau") == True; _ledger.append(1)
assert hasattr(cmath, "isfinite") == True; _ledger.append(1)
assert hasattr(cmath, "isinf") == True; _ledger.append(1)
assert hasattr(cmath, "isnan") == True; _ledger.append(1)
assert hasattr(cmath, "polar") == True; _ledger.append(1)
assert hasattr(cmath, "rect") == True; _ledger.append(1)
assert hasattr(cmath, "phase") == True; _ledger.append(1)

# 2) cmath — value contracts (conformant subset)
assert cmath.sqrt(-1) == 1j; _ledger.append(1)
assert cmath.isnan(1 + 1j) == False; _ledger.append(1)

# 3) difflib — hasattr (conformant subset) + value contract
assert hasattr(difflib, "SequenceMatcher") == True; _ledger.append(1)
assert hasattr(difflib, "unified_diff") == True; _ledger.append(1)
assert hasattr(difflib, "get_close_matches") == True; _ledger.append(1)
assert difflib.get_close_matches("hello", ["help", "hellp", "world"]) == ["hellp", "help"]; _ledger.append(1)

# 4) shlex — hasattr (conformant subset) + value contracts
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.quote("a b") == "'a b'"; _ledger.append(1)
assert shlex.join(["a", "b", "c"]) == "a b c"; _ledger.append(1)

# 5) configparser — hasattr ConfigParser only (conformant subset)
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

# 6) csv — hasattr core surface
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "unix_dialect") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)
assert hasattr(csv, "field_size_limit") == True; _ledger.append(1)

# 7) secrets — hasattr core surface + value contracts
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "randbits") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
assert secrets.compare_digest("abc", "abd") == False; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_cmath_difflib_shlex_csv_secrets_value_ops {sum(_ledger)} asserts")
