# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlencode_doseq_expands_lists"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode renders a mapping into an &-joined query, quote_plus-escaping spaces; with doseq=True a list value expands into one key=value pair per element"""
from urllib.parse import urlencode

enc = urlencode({"a": 1, "b": "hello world"})
assert "a=1" in enc, f"urlencode a=1 = {enc!r}"
assert "b=hello+world" in enc, f"urlencode quote_plus = {enc!r}"

params = {"colors": ["red", "green", "blue"], "count": 3}
seq = urlencode(params, doseq=True)
assert set(seq.split("&")) == {"colors=red", "colors=green", "colors=blue", "count=3"}, f"doseq = {seq!r}"

print("urlencode_doseq_expands_lists OK")
