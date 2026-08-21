# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "template_static_substitute_no_self_raises"
# subject = "string.Template"
# kind = "mechanical"
# xfail = "string.Template is a silent dict-stub on mamba; unbound .substitute() does not raise (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: template_static_substitute_no_self_raises (errors)."""
import string

_raised = False
try:
    string.Template.substitute()
except TypeError:
    _raised = True
assert _raised, "template_static_substitute_no_self_raises: expected TypeError"
print("template_static_substitute_no_self_raises OK")
