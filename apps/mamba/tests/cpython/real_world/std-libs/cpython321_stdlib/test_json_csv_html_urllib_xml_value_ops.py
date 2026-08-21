# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json_csv_html_urllib_xml_value_ops"
# subject = "cpython321.test_json_csv_html_urllib_xml_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json_csv_html_urllib_xml_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_json_csv_html_urllib_xml_value_ops: execute CPython 3.12 seed test_json_csv_html_urllib_xml_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 296 pass conformance — json module (hasattr dumps/loads/
# dump/load/JSONEncoder/JSONDecoder/JSONDecodeError + dumps/loads
# round-trip for dict/list/str/None/True/1.5 + sort_keys + indent) +
# csv module (hasattr reader/writer/DictReader/DictWriter/Dialect/
# excel/excel_tab/unix_dialect/QUOTE_ALL/QUOTE_MINIMAL/QUOTE_NONE/
# QUOTE_NONNUMERIC/Error/field_size_limit/list_dialects/register_
# dialect/unregister_dialect/get_dialect + QUOTE constant values) +
# html module (hasattr escape/unescape + escape/unescape contracts)
# + urllib.parse (urlparse scheme/netloc/path + urlunparse round-
# trip + quote/unquote + parse_qs/parse_qsl/urlencode).
# All asserts match between CPython 3.12 and mamba.
import json
import csv
import html
from urllib.parse import urlparse, urlunparse, quote, unquote, parse_qs, parse_qsl, urlencode


_ledger: list[int] = []

# 1) json — hasattr core surface
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)

# 2) json — value contracts (dumps/loads)
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.loads('{"a":1}') == {"a": 1}; _ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps("hi") == '"hi"'; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps(1.5) == "1.5"; _ledger.append(1)
assert isinstance(json.dumps({}), str) == True; _ledger.append(1)
assert isinstance(json.loads("[]"), list) == True; _ledger.append(1)
assert isinstance(json.loads("{}"), dict) == True; _ledger.append(1)
assert json.dumps({"a": 1, "b": 2}, sort_keys=True) == '{"a": 1, "b": 2}'; _ledger.append(1)
assert json.dumps([1, 2], indent=2) == "[\n  1,\n  2\n]"; _ledger.append(1)
assert json.loads(json.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)

# 3) csv — hasattr core surface
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
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)

# 4) csv — QUOTE constant values
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)

# 5) html — hasattr core surface
assert hasattr(html, "escape") == True; _ledger.append(1)
assert hasattr(html, "unescape") == True; _ledger.append(1)

# 6) html — value contracts (named entity coverage)
assert html.escape("<a&b>") == "&lt;a&amp;b&gt;"; _ledger.append(1)
assert html.escape('a"b\'c') == "a&quot;b&#x27;c"; _ledger.append(1)
assert html.unescape("&lt;a&gt;") == "<a>"; _ledger.append(1)
assert html.unescape("&amp;") == "&"; _ledger.append(1)

# 7) urllib.parse — value contracts
assert urlparse("http://x/y").scheme == "http"; _ledger.append(1)
assert urlparse("http://x/y").netloc == "x"; _ledger.append(1)
assert urlparse("http://x/y").path == "/y"; _ledger.append(1)
assert urlunparse(urlparse("http://x/y")) == "http://x/y"; _ledger.append(1)
assert quote("a b") == "a%20b"; _ledger.append(1)
assert quote("a/b") == "a/b"; _ledger.append(1)
assert unquote("a%20b") == "a b"; _ledger.append(1)
assert parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)
assert urlencode({"a": 1, "b": 2}) == "a=1&b=2"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_csv_html_urllib_xml_value_ops {sum(_ledger)} asserts")
