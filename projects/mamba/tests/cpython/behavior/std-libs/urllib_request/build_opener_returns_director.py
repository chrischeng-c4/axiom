# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "behavior"
# case = "build_opener_returns_director"
# subject = "urllib.request.build_opener"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: build_opener returns a dict, not an OpenerDirector (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.build_opener: build_opener() returns an OpenerDirector instance (the default opener used by urlopen)"""
from urllib.request import build_opener, OpenerDirector

opener = build_opener()
assert isinstance(opener, OpenerDirector), f"build_opener() type = {type(opener).__name__!r}"
assert type(opener).__name__ == "OpenerDirector", f"name = {type(opener).__name__!r}"

print("build_opener_returns_director OK")
