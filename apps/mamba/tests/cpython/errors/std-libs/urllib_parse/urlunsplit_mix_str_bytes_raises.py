# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "urlunsplit_mix_str_bytes_raises"
# subject = "urllib.parse.urlunsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlunsplit: urlunsplit_mix_str_bytes_raises (errors)."""
from urllib.parse import urlunsplit

_raised = False
try:
    urlunsplit(('http', b'h', '', '', ''))
except TypeError:
    _raised = True
assert _raised, "urlunsplit_mix_str_bytes_raises: expected TypeError"
print("urlunsplit_mix_str_bytes_raises OK")
