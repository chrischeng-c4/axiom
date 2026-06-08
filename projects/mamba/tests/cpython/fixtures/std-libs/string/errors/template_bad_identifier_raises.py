# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "template_bad_identifier_raises"
# subject = "string.Template"
# kind = "mechanical"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: template_bad_identifier_raises (errors)."""
import string

_raised = False
try:
    string.Template('$0_bad').substitute({'0_bad': 'value'})
except ValueError:
    _raised = True
assert _raised, "template_bad_identifier_raises: expected ValueError"
print("template_bad_identifier_raises OK")
