# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "urlencode_bare_str_raises"
# subject = "urllib.parse.urlencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode_bare_str_raises (errors)."""
from urllib.parse import urlencode

_raised = False
try:
    urlencode('foo')
except TypeError:
    _raised = True
assert _raised, "urlencode_bare_str_raises: expected TypeError"
print("urlencode_bare_str_raises OK")
