# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_os_path_dataclasses_email_decimal_silent"
# subject = "cpython321.lang_os_path_dataclasses_email_decimal_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_os_path_dataclasses_email_decimal_silent.py"
# status = "filled"
# ///
"""cpython321.lang_os_path_dataclasses_email_decimal_silent: execute CPython 3.12 seed lang_os_path_dataclasses_email_decimal_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# pure-Python / utility quintet pinned by atomic 142: `os.path`
# (the documented isabs / normpath module-level helpers and
# pathsep / curdir / pardir / devnull string sentinels),
# `dataclasses` (the MISSING sentinel, the Field class identity,
# and the is_dataclass type predicate), `email.utils` (the
# parseaddr / formataddr address splitter / formatter and the
# quote / unquote / parsedate helpers), `contextvars` (the
# ContextVar / Context bare class identity), and `decimal` (the
# ROUND_HALF_UP / ROUND_HALF_EVEN / ROUND_DOWN / ROUND_UP rounding-
# mode string sentinels and Decimal / Context / InvalidOperation /
# DivisionByZero bare class identity).
#
# The matching subset (os.path.join / split / splitext / basename /
# dirname / sep / altsep + exists / isdir / isfile inspectors +
# expanduser non-tilde-passthrough) is covered by
# `test_os_path_filesystem_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • os.path.isabs("/foo") is True — POSIX absolute-path predicate
#     (mamba: returns None);
#   • os.path.isabs("foo") is False (mamba: None);
#   • os.path.normpath("a/b/../c") == "a/c" — collapses "..">
#     (mamba: None);
#   • os.path.pathsep == ":" — POSIX PATH-separator (mamba: None);
#   • os.path.curdir == "." — current-dir sentinel (mamba: None);
#   • os.path.pardir == ".." — parent-dir sentinel (mamba: None);
#   • os.path.devnull == "/dev/null" — POSIX null-device sentinel
#     (mamba: None);
#   • dataclasses.MISSING — sentinel singleton (mamba: None);
#   • type(dataclasses.MISSING).__name__ == "_MISSING_TYPE" (mamba:
#     "NoneType");
#   • dataclasses.Field.__name__ == "Field" — class identity
#     (mamba: None);
#   • dataclasses.is_dataclass(42) is False — type predicate
#     (mamba: AttributeError, the binding does not exist);
#   • email.utils.parseaddr("Alice <alice@example.com>") ==
#     ("Alice", "alice@example.com") — addr-spec split
#     (mamba: returns ['', '']);
#   • email.utils.formataddr(("Alice", "alice@example.com")) ==
#     "Alice <alice@example.com>" — addr formatter (mamba:
#     returns empty string);
#   • email.utils.quote("hello") == "hello" — header-value quote
#     identity (mamba: returns empty string);
#   • email.utils.unquote("\"hello\"") == "hello" — header-value
#     unquote (mamba: returns empty string);
#   • email.utils.parsedate("Sun, 06 Nov 1994 08:49:37 GMT") ==
#     (1994, 11, 6, 8, 49, 37, 0, 1, -1) (mamba: returns []);
#   • contextvars.ContextVar.__name__ == "ContextVar" — class
#     identity (mamba: None);
#   • contextvars.Context.__name__ == "Context" (mamba: None);
#   • decimal.ROUND_HALF_UP == "ROUND_HALF_UP" — documented
#     rounding-mode string sentinel (mamba: None);
#   • decimal.ROUND_HALF_EVEN == "ROUND_HALF_EVEN" (mamba: None);
#   • decimal.ROUND_DOWN == "ROUND_DOWN" (mamba: None);
#   • decimal.ROUND_UP == "ROUND_UP" (mamba: None);
#   • decimal.Decimal.__name__ == "Decimal" — class identity
#     (mamba: None);
#   • decimal.Context.__name__ == "Context" (mamba: None);
#   • decimal.InvalidOperation.__name__ == "InvalidOperation"
#     (mamba: None);
#   • decimal.DivisionByZero.__name__ == "DivisionByZero" (mamba:
#     None).
import os as _os_mod
import dataclasses as _dc_mod
import email.utils as _eu_mod
import contextvars as _cv_mod
import decimal as _dec_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constants / class identifiers / module-level helpers that
# mamba's bundled type stubs do not surface accurately.
os: Any = _os_mod
dataclasses: Any = _dc_mod
eu: Any = _eu_mod
contextvars: Any = _cv_mod
decimal: Any = _dec_mod

_ledger: list[int] = []

# 1) os.path.isabs — POSIX absolute-path predicate
assert os.path.isabs("/foo") == True; _ledger.append(1)
assert os.path.isabs("foo") == False; _ledger.append(1)

# 2) os.path.normpath — collapses ".." segments
assert os.path.normpath("a/b/../c") == "a/c"; _ledger.append(1)

# 3) os.path.pathsep / curdir / pardir / devnull — POSIX sentinels
assert os.path.pathsep == ":"; _ledger.append(1)
assert os.path.curdir == "."; _ledger.append(1)
assert os.path.pardir == ".."; _ledger.append(1)
assert os.path.devnull == "/dev/null"; _ledger.append(1)

# 4) dataclasses.MISSING — sentinel singleton (non-None)
assert dataclasses.MISSING is not None; _ledger.append(1)
assert type(dataclasses.MISSING).__name__ == "_MISSING_TYPE"; _ledger.append(1)

# 5) dataclasses.Field — bare class-name identity
assert dataclasses.Field.__name__ == "Field"; _ledger.append(1)

# 6) dataclasses.is_dataclass — type predicate
assert dataclasses.is_dataclass(42) == False; _ledger.append(1)
assert dataclasses.is_dataclass("hi") == False; _ledger.append(1)

# 7) email.utils.parseaddr — addr-spec split
assert eu.parseaddr("Alice <alice@example.com>") == ("Alice", "alice@example.com"); _ledger.append(1)
assert eu.parseaddr("plain@example.com") == ("", "plain@example.com"); _ledger.append(1)

# 8) email.utils.formataddr — addr-spec formatter
assert eu.formataddr(("Alice", "alice@example.com")) == "Alice <alice@example.com>"; _ledger.append(1)

# 9) email.utils.quote / unquote — header-value helpers
assert eu.quote("hello") == "hello"; _ledger.append(1)
assert eu.unquote('"hello"') == "hello"; _ledger.append(1)

# 10) email.utils.parsedate — RFC 2822 date tuple
assert eu.parsedate("Sun, 06 Nov 1994 08:49:37 GMT") == (1994, 11, 6, 8, 49, 37, 0, 1, -1); _ledger.append(1)

# 11) contextvars — bare class identity
assert contextvars.ContextVar.__name__ == "ContextVar"; _ledger.append(1)
assert contextvars.Context.__name__ == "Context"; _ledger.append(1)

# 12) decimal — rounding-mode string sentinels
assert decimal.ROUND_HALF_UP == "ROUND_HALF_UP"; _ledger.append(1)
assert decimal.ROUND_HALF_EVEN == "ROUND_HALF_EVEN"; _ledger.append(1)
assert decimal.ROUND_DOWN == "ROUND_DOWN"; _ledger.append(1)
assert decimal.ROUND_UP == "ROUND_UP"; _ledger.append(1)

# 13) decimal — bare class identity
assert decimal.Decimal.__name__ == "Decimal"; _ledger.append(1)
assert decimal.Context.__name__ == "Context"; _ledger.append(1)
assert decimal.InvalidOperation.__name__ == "InvalidOperation"; _ledger.append(1)
assert decimal.DivisionByZero.__name__ == "DivisionByZero"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_os_path_dataclasses_email_decimal_silent {sum(_ledger)} asserts")
