# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_bytes_returns_bytes"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter with bytes names + bytes pattern keeps the matching bytes objects (every result element is bytes); filter(['Python',...], 'P*') with str is unaffected"""
import fnmatch

# filter with bytes names + bytes pattern keeps the matching bytes objects.
_b = fnmatch.filter([b"Python", b"Ruby", b"Perl", b"Tcl"], b"P*")
assert _b == [b"Python", b"Perl"], f"bytes filter = {_b!r}"
assert all(isinstance(x, bytes) for x in _b), "filter returns bytes elements"

# The str path is unaffected by the bytes path.
_s = fnmatch.filter(["Python", "Ruby", "Perl", "Tcl"], "P*")
assert _s == ["Python", "Perl"], f"str filter = {_s!r}"

print("filter_bytes_returns_bytes OK")
