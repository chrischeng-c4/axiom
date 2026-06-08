# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "ensure_ascii_escaping"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_unicode.py"
# status = "filled"
# ///
"""json.dumps: non-ASCII characters are \\uXXXX-escaped by default but emitted verbatim when ensure_ascii=False; both round-trip"""
import json

escaped = json.dumps("café")
assert escaped == '"caf\\u00e9"', f"escaped = {escaped!r}"
assert json.loads(escaped) == "café", "escaped unicode round-trip"

raw = json.dumps({"key": "café"}, ensure_ascii=False)
assert "café" in raw, f"unicode verbatim = {raw!r}"
assert json.loads(raw) == {"key": "café"}, "verbatim unicode round-trip"

print("ensure_ascii_escaping OK")
