# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib modules used by every observability /
# CLI / config / tabular-data / XML / embedded-db path:
# `logging` (the documented severity-level constants + module-
# level severity-shortcut helpers + getLogger surface),
# `argparse` (the ArgumentParser bare-class surface), `csv`
# (the QUOTE_* constants + the documented reader / writer /
# DictReader / DictWriter surface), `sqlite3` (the
# PARSE_DECLTYPES / PARSE_COLNAMES constants + the documented
# connect helper), `xml.etree.ElementTree` (the Element /
# SubElement / fromstring / tostring documented surface +
# the root-tag round-trip), and `configparser` (the
# ConfigParser bare-class surface).
#
# The matching subset between mamba and CPython is the level-
# constant layer + module-level helper layer + tabular-data
# layer + embedded-db connect layer + XML round-trip layer:
# logging.DEBUG / INFO / WARNING / ERROR / CRITICAL resolve to
# their documented severity integers; logging.getLogger /
# basicConfig / debug / info / warning / error / critical exist;
# csv.QUOTE_MINIMAL / QUOTE_ALL / QUOTE_NONNUMERIC / QUOTE_NONE
# resolve to 0/1/2/3; csv.writer / reader / DictReader /
# DictWriter exist; sqlite3.PARSE_DECLTYPES / PARSE_COLNAMES
# resolve to 1/2; sqlite3.connect exists; ET.Element /
# SubElement / fromstring / tostring exist; ET.fromstring("<root>
# ...</root>").tag == "root"; logging.getLogger("name").name ==
# "name".
#
# Surface in this fixture:
#   • logging.DEBUG / INFO / WARNING / ERROR / CRITICAL — 10 /
#     20 / 30 / 40 / 50;
#   • logging hasattr — getLogger / basicConfig + the severity-
#     shortcut helpers debug / info / warning / error / critical;
#   • logging.getLogger("test").name == "test";
#   • argparse — hasattr ArgumentParser;
#   • configparser — hasattr ConfigParser;
#   • csv.QUOTE_MINIMAL / QUOTE_ALL / QUOTE_NONNUMERIC /
#     QUOTE_NONE constants;
#   • csv — hasattr writer / reader / DictReader / DictWriter;
#   • sqlite3.PARSE_DECLTYPES / PARSE_COLNAMES constants;
#   • sqlite3 — hasattr connect;
#   • xml.etree.ElementTree — hasattr Element / SubElement /
#     fromstring / tostring;
#   • ET.fromstring("<root>...</root>").tag == "root".
#
# Behavioral edges that DIVERGE on mamba (logging.NOTSET None vs
# 0, logging.Logger / Handler / Formatter / StreamHandler /
# LogRecord class identity, logging.getLogger(...).level None vs
# 0, logging.getLevelName AttributeError, argparse.Namespace /
# ArgumentError / SUPPRESS hasattr False, configparser.Error /
# NoSectionError / Interpolation hasattr False,
# ConfigParser instance .sections() broken,
# sqlite3.Connection / Cursor / OperationalError hasattr False,
# sqlite3.connect(":memory:").cursor() broken,
# ET.fromstring(...)[0] KeyError) are covered in the matching
# spec fixture `lang_logging_sqlite_etree_silent`.
import logging
import argparse
import configparser
import csv
import sqlite3
import xml.etree.ElementTree as ET


_ledger: list[int] = []

# 1) logging — severity-level constants
assert logging.DEBUG == 10; _ledger.append(1)
assert logging.INFO == 20; _ledger.append(1)
assert logging.WARNING == 30; _ledger.append(1)
assert logging.ERROR == 40; _ledger.append(1)
assert logging.CRITICAL == 50; _ledger.append(1)

# 2) logging hasattr — module-level helpers + severity shortcuts
assert hasattr(logging, "getLogger"); _ledger.append(1)
assert hasattr(logging, "basicConfig"); _ledger.append(1)
assert hasattr(logging, "debug"); _ledger.append(1)
assert hasattr(logging, "info"); _ledger.append(1)
assert hasattr(logging, "warning"); _ledger.append(1)
assert hasattr(logging, "error"); _ledger.append(1)
assert hasattr(logging, "critical"); _ledger.append(1)

# 3) logging.getLogger — name round-trip
assert logging.getLogger("test").name == "test"; _ledger.append(1)

# 4) argparse — ArgumentParser surface
assert hasattr(argparse, "ArgumentParser"); _ledger.append(1)

# 5) configparser — ConfigParser surface
assert hasattr(configparser, "ConfigParser"); _ledger.append(1)

# 6) csv — QUOTE_* constants
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)

# 7) csv hasattr — reader / writer + dict variants
assert hasattr(csv, "writer"); _ledger.append(1)
assert hasattr(csv, "reader"); _ledger.append(1)
assert hasattr(csv, "DictReader"); _ledger.append(1)
assert hasattr(csv, "DictWriter"); _ledger.append(1)

# 8) sqlite3 — type-detection flags
assert sqlite3.PARSE_DECLTYPES == 1; _ledger.append(1)
assert sqlite3.PARSE_COLNAMES == 2; _ledger.append(1)

# 9) sqlite3 — connect helper surface
assert hasattr(sqlite3, "connect"); _ledger.append(1)

# 10) xml.etree.ElementTree — public surface
assert hasattr(ET, "Element"); _ledger.append(1)
assert hasattr(ET, "SubElement"); _ledger.append(1)
assert hasattr(ET, "fromstring"); _ledger.append(1)
assert hasattr(ET, "tostring"); _ledger.append(1)

# 11) xml.etree.ElementTree — fromstring round-trip on root tag
assert ET.fromstring("<root><child attr='val'>hello</child></root>").tag == "root"; _ledger.append(1)

# NB: logging.NOTSET None vs 0, logging.Logger / Handler /
# Formatter / StreamHandler / LogRecord class identity,
# logging.getLogger(...).level None vs 0,
# logging.getLevelName AttributeError, argparse.Namespace /
# ArgumentError / SUPPRESS hasattr False, configparser.Error /
# NoSectionError / Interpolation hasattr False,
# ConfigParser instance .sections() broken,
# sqlite3.Connection / Cursor / OperationalError hasattr False,
# sqlite3.connect(":memory:").cursor() broken,
# ET.fromstring(...)[0] KeyError all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_logging_csv_etree_surface_value_ops {sum(_ledger)} asserts")
