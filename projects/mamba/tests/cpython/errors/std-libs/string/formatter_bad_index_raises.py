# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "formatter_bad_index_raises"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = "string.Formatter is a silent dict-stub on mamba; .vformat() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Formatter: formatter_bad_index_raises (errors)."""
import string

_raised = False
try:
    string.Formatter().vformat('{5}', [1, 2], {})
except IndexError:
    _raised = True
assert _raised, "formatter_bad_index_raises: expected IndexError"
print("formatter_bad_index_raises OK")
