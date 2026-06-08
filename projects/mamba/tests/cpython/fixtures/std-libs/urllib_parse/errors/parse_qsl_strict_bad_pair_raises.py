# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "errors"
# case = "parse_qsl_strict_bad_pair_raises"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_strict_bad_pair_raises (errors)."""
from urllib.parse import parse_qsl

_raised = False
try:
    parse_qsl('novalkey', strict_parsing=True)
except ValueError:
    _raised = True
assert _raised, "parse_qsl_strict_bad_pair_raises: expected ValueError"
print("parse_qsl_strict_bad_pair_raises OK")
