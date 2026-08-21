# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "port_out_of_range_raises"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: port_out_of_range_raises (errors)."""
from urllib.parse import urlsplit

_raised = False
try:
    urlsplit('http://host:65536/doc/').port
except ValueError:
    _raised = True
assert _raised, "port_out_of_range_raises: expected ValueError"
print("port_out_of_range_raises OK")
