# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "parse_qsl_max_num_fields_raises"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_max_num_fields_raises (errors)."""
from urllib.parse import parse_qsl

_raised = False
try:
    parse_qsl('&'.join(['a=a'] * 11), max_num_fields=10)
except ValueError:
    _raised = True
assert _raised, "parse_qsl_max_num_fields_raises: expected ValueError"
print("parse_qsl_max_num_fields_raises OK")
