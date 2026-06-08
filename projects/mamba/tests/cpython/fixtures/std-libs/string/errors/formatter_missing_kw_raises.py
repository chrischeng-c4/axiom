# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_missing_kw_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .vformat() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_missing_kw_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().vformat('{name}', [], {})
except KeyError:
    _raised = True
assert _raised, "formatter_missing_kw_raises: expected KeyError"
print("formatter_missing_kw_raises OK")
