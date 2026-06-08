# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "bad_regex_raises"
# subject = "warnings.filterwarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: bad_regex_raises (errors)."""
import re
import warnings

_raised = False
try:
    warnings.filterwarnings("ignore", message="*(")
except re.error:
    _raised = True
assert _raised, "bad_regex_raises: expected re.error"
print("bad_regex_raises OK")
