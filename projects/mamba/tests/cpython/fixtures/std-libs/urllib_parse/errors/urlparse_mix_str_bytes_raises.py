# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "urlparse_mix_str_bytes_raises"
# subject = "urllib.parse.urlparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse_mix_str_bytes_raises (errors)."""
from urllib.parse import urlparse

_raised = False
try:
    urlparse('www.python.org', b'http')
except TypeError:
    _raised = True
assert _raised, "urlparse_mix_str_bytes_raises: expected TypeError"
print("urlparse_mix_str_bytes_raises OK")
