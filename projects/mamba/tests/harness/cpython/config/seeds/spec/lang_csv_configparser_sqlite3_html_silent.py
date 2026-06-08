# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(csv, 'Sniffer')` (the
# documented "csv exposes the dialect-Sniffer class" — mamba returns
# False), `type(csv.reader(...)).__name__ == 'reader'` (the
# documented "csv.reader returns a streaming reader iterator" —
# mamba returns 'list' — eager list materialization),
# `hasattr(configparser, 'DEFAULTSECT')` (the documented
# "configparser exposes the DEFAULTSECT 'DEFAULT' constant" — mamba
# returns False), `hasattr(configparser, 'NoSectionError')` (the
# documented "configparser exposes the NoSectionError exception" —
# mamba returns False), `type(configparser.ConfigParser()).__name__
# == 'ConfigParser'` (the documented "ConfigParser() returns a
# ConfigParser instance" — mamba returns 'dict' — constructor
# degrades to a plain dict), `hasattr(sqlite3, 'Connection')` (the
# documented "sqlite3 exposes the Connection class" — mamba returns
# False), `type(sqlite3.connect(':memory:')).__name__ ==
# 'Connection'` (the documented "sqlite3.connect returns a
# Connection instance" — mamba returns 'dict' — constructor
# degrades to a plain dict),
# `type(xml.etree.ElementTree.Element('a')).__name__ == 'Element'`
# (the documented "ET.Element returns an Element instance" — mamba
# returns 'dict' — constructor degrades to a plain dict),
# `html.unescape('&#65;') == 'A'` (the documented "html.unescape
# decodes numeric character references" — mamba returns '&#65;' —
# numeric references pass through unchanged), and `isinstance(xml.
# etree.ElementTree.tostring(xml.etree.ElementTree.Element('a')),
# bytes)` (the documented "ET.tostring returns bytes by default" —
# mamba returns str — output type drifts from bytes to str).
# Ten-pack pinned to atomic 296.
#
# Behavioral edges that CONFORM on mamba (json — hasattr dumps/
# loads/dump/load/JSONEncoder/JSONDecoder/JSONDecodeError + dumps/
# loads round-trips. csv — hasattr reader/writer/DictReader/
# DictWriter/Dialect/excel/excel_tab/unix_dialect/QUOTE_ALL/QUOTE_
# MINIMAL/QUOTE_NONE/QUOTE_NONNUMERIC/Error/field_size_limit/list_
# dialects/register_dialect/unregister_dialect/get_dialect + QUOTE
# constant values. html — hasattr escape/unescape + named-entity
# escape/unescape. urllib.parse — urlparse scheme/netloc/path +
# urlunparse + quote/unquote + parse_qs/qsl/urlencode) are covered
# in the matching pass fixture `test_json_csv_html_urllib_xml_value
# _ops`.
import csv
import configparser
import sqlite3
import xml.etree.ElementTree as ET
import html


_ledger: list[int] = []

# 1) hasattr(csv, 'Sniffer') — dialect-Sniffer class
#    (mamba: returns False)
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 2) type(csv.reader(...)).__name__ == 'reader' — streaming reader iterator
#    (mamba: returns 'list' — eager list materialization)
import io
assert type(csv.reader(io.StringIO("a,b"))).__name__ == "reader"; _ledger.append(1)

# 3) hasattr(configparser, 'DEFAULTSECT') — DEFAULTSECT 'DEFAULT' constant
#    (mamba: returns False)
assert hasattr(configparser, "DEFAULTSECT") == True; _ledger.append(1)

# 4) hasattr(configparser, 'NoSectionError') — NoSectionError exception
#    (mamba: returns False)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)

# 5) type(configparser.ConfigParser()).__name__ == 'ConfigParser' — ConfigParser instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(configparser.ConfigParser()).__name__ == "ConfigParser"; _ledger.append(1)

# 6) hasattr(sqlite3, 'Connection') — Connection class
#    (mamba: returns False)
assert hasattr(sqlite3, "Connection") == True; _ledger.append(1)

# 7) type(sqlite3.connect(':memory:')).__name__ == 'Connection' — Connection instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(sqlite3.connect(":memory:")).__name__ == "Connection"; _ledger.append(1)

# 8) type(ET.Element('a')).__name__ == 'Element' — Element instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(ET.Element("a")).__name__ == "Element"; _ledger.append(1)

# 9) html.unescape('&#65;') == 'A' — numeric character reference decoded
#    (mamba: returns '&#65;' — numeric reference unchanged)
assert html.unescape("&#65;") == "A"; _ledger.append(1)

# 10) isinstance(ET.tostring(ET.Element('a')), bytes) — tostring returns bytes by default
#     (mamba: returns str — output type drifts from bytes to str)
assert isinstance(ET.tostring(ET.Element("a")), bytes) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_csv_configparser_sqlite3_html_silent {sum(_ledger)} asserts")
