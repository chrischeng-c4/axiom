# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "real_world"
# case = "build_and_parse_query_url"
# subject = "urllib.parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse: build a query URL with urlencode, urlparse it back out, and parse_qs the query to recover the original parameter mapping (a real client request-building round-trip)"""
from urllib.parse import urlencode, urlparse, parse_qs

params = {"q": "open source", "page": "2", "lang": "en"}
query = urlencode(params)
url = "https://search.example.com/results?" + query
parsed = urlparse(url)
assert parsed.scheme == "https", parsed.scheme
assert parsed.netloc == "search.example.com", parsed.netloc
assert parsed.path == "/results", parsed.path
recovered = parse_qs(parsed.query)
assert recovered == {"q": ["open source"], "page": ["2"], "lang": ["en"]}, recovered

print("build_and_parse_query_url OK")
