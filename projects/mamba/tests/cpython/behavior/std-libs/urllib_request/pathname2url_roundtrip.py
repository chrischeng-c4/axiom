# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "pathname2url_roundtrip"
# subject = "urllib.request.pathname2url"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: pathname2url returns {} (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.pathname2url: pathname2url returns a str and url2pathname recovers the original POSIX path (round-trip of an absolute path)"""
from urllib.request import pathname2url, url2pathname

url = pathname2url("/usr/local/bin")
assert isinstance(url, str), f"pathname2url type = {type(url).__name__!r}"
assert url2pathname(url) == "/usr/local/bin", f"round-trip = {url2pathname(url)!r}"

print("pathname2url_roundtrip OK")
