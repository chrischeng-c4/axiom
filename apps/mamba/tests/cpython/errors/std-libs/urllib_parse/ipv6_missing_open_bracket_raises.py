# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "ipv6_missing_open_bracket_raises"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: ipv6_missing_open_bracket_raises (errors)."""
from urllib.parse import urlsplit

_raised = False
try:
    urlsplit('scheme://v6a.ip]')
except ValueError:
    _raised = True
assert _raised, "ipv6_missing_open_bracket_raises: expected ValueError"
print("ipv6_missing_open_bracket_raises OK")
