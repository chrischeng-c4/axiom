# Operational AssertionPass seed for SILENT divergences across the
# observability / CLI / config / embedded-DB / XML-tree quintet
# pinned by atomic 152: `logging` (the documented NOTSET == 0
# severity constant + the Logger / Handler / Formatter /
# StreamHandler / LogRecord class identity + the
# Logger().level == 0 default + the documented
# `getLevelName` helper), `argparse` (the documented Namespace
# / ArgumentError class identity + SUPPRESS sentinel),
# `configparser` (the documented NoSectionError / Error /
# Interpolation class identity + ConfigParser instance
# `.sections()` ConfigParser-instance behaviour), `sqlite3`
# (the documented Connection / Cursor / OperationalError class
# identity + the `connect(":memory:").cursor()` instance
# method), and `xml.etree.ElementTree` (the documented Element
# index-subscript `root[0]` access).
#
# The matching subset (logging.DEBUG / INFO / WARNING / ERROR /
# CRITICAL severity-level constants + logging hasattr getLogger
# / basicConfig / debug / info / warning / error / critical +
# logging.getLogger("test").name; argparse / configparser
# hasattr ArgumentParser / ConfigParser; csv.QUOTE_* constants
# + writer / reader / DictReader / DictWriter hasattr;
# sqlite3.PARSE_DECLTYPES / PARSE_COLNAMES + sqlite3.connect
# hasattr; xml.etree.ElementTree Element / SubElement /
# fromstring / tostring hasattr + fromstring tag round-trip)
# is covered by `test_logging_csv_etree_surface_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • logging.NOTSET == 0 — the documented unset-severity
#     sentinel (mamba: returns None, NOTSET is not surfaced);
#   • logging.Logger.__name__ == "Logger" — bare class
#     identity (mamba: hasattr returns False);
#   • logging.Handler.__name__ == "Handler" (mamba: hasattr
#     returns False);
#   • logging.Formatter.__name__ == "Formatter" (mamba:
#     hasattr returns False);
#   • logging.StreamHandler.__name__ == "StreamHandler" (mamba:
#     hasattr returns False);
#   • logging.LogRecord.__name__ == "LogRecord" (mamba:
#     hasattr returns False);
#   • logging.getLogger("test").level == 0 — the documented
#     unset-level default (mamba: returns None);
#   • logging.getLevelName(logging.INFO) == "INFO" — the
#     documented level→name helper (mamba: AttributeError,
#     'dict' object has no attribute 'getLevelName');
#   • argparse.Namespace.__name__ == "Namespace" — namespace
#     class identity (mamba: hasattr returns False);
#   • argparse.ArgumentError.__name__ == "ArgumentError" (mamba:
#     hasattr returns False);
#   • hasattr(argparse, "SUPPRESS") is True — the documented
#     argument-suppression sentinel (mamba: False);
#   • configparser.NoSectionError.__name__ == "NoSectionError"
#     (mamba: hasattr returns False);
#   • configparser.Error.__name__ == "Error" (mamba: hasattr
#     returns False);
#   • hasattr(configparser, "Interpolation") is True (mamba:
#     False);
#   • configparser.ConfigParser().sections() == [] — sections
#     accessor on an empty parser (mamba: AttributeError,
#     'dict' object has no attribute 'sections');
#   • sqlite3.Connection.__name__ == "Connection" — DB-API
#     class identity (mamba: hasattr returns False);
#   • sqlite3.Cursor.__name__ == "Cursor" (mamba: hasattr
#     returns False);
#   • sqlite3.OperationalError.__name__ == "OperationalError"
#     (mamba: hasattr returns False);
#   • sqlite3.connect(":memory:").cursor() — exposes the
#     documented cursor method on a live Connection (mamba:
#     AttributeError, 'dict' object has no attribute 'cursor');
#   • ET.fromstring("<root><child/></root>")[0].tag == "child"
#     — Element subscript access (mamba: KeyError '0', mamba
#     does not implement subscript on Element).
import logging as _logging_mod
import argparse as _argparse_mod
import configparser as _configparser_mod
import sqlite3 as _sqlite3_mod
import xml.etree.ElementTree as _ET_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
logging: Any = _logging_mod
argparse: Any = _argparse_mod
configparser: Any = _configparser_mod
sqlite3: Any = _sqlite3_mod
ET: Any = _ET_mod


_ledger: list[int] = []

# 1) logging.NOTSET — unset-severity sentinel
assert logging.NOTSET == 0; _ledger.append(1)

# 2) logging — class identity surface
assert logging.Logger.__name__ == "Logger"; _ledger.append(1)
assert logging.Handler.__name__ == "Handler"; _ledger.append(1)
assert logging.Formatter.__name__ == "Formatter"; _ledger.append(1)
assert logging.StreamHandler.__name__ == "StreamHandler"; _ledger.append(1)
assert logging.LogRecord.__name__ == "LogRecord"; _ledger.append(1)

# 3) logging.getLogger — default unset-level
assert logging.getLogger("test").level == 0; _ledger.append(1)

# 4) logging.getLevelName — documented level→name helper
assert logging.getLevelName(logging.INFO) == "INFO"; _ledger.append(1)

# 5) argparse — class identity + SUPPRESS sentinel
assert argparse.Namespace.__name__ == "Namespace"; _ledger.append(1)
assert argparse.ArgumentError.__name__ == "ArgumentError"; _ledger.append(1)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)

# 6) configparser — class identity surface
assert configparser.NoSectionError.__name__ == "NoSectionError"; _ledger.append(1)
assert configparser.Error.__name__ == "Error"; _ledger.append(1)
assert hasattr(configparser, "Interpolation") == True; _ledger.append(1)

# 7) configparser.ConfigParser — empty-instance sections accessor
assert configparser.ConfigParser().sections() == []; _ledger.append(1)

# 8) sqlite3 — DB-API class identity surface
assert sqlite3.Connection.__name__ == "Connection"; _ledger.append(1)
assert sqlite3.Cursor.__name__ == "Cursor"; _ledger.append(1)
assert sqlite3.OperationalError.__name__ == "OperationalError"; _ledger.append(1)

# 9) sqlite3 — connect(":memory:").cursor instance method
_conn = sqlite3.connect(":memory:")
_cur = _conn.cursor()
assert _cur is not None; _ledger.append(1)
_conn.close()

# 10) xml.etree.ElementTree — Element subscript access
assert ET.fromstring("<root><child/></root>")[0].tag == "child"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_logging_sqlite_etree_silent {sum(_ledger)} asserts")
