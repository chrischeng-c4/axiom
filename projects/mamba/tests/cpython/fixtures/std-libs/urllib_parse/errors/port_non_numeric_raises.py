# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "port_non_numeric_raises"
# subject = "urllib.parse.urlparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: port_non_numeric_raises (errors)."""
from urllib.parse import urlparse

_raised = False
try:
    urlparse('http://Server=sde; Service=sde:oracle').port
except ValueError:
    _raised = True
assert _raised, "port_non_numeric_raises: expected ValueError"
print("port_non_numeric_raises OK")
