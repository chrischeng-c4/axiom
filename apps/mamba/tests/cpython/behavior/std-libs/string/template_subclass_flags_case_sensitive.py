# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_subclass_flags_case_sensitive"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template subclassing relies on the substitution engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a Template subclass setting `flags = 0` disables the default re.IGNORECASE so a mixed-case '$wHO' becomes an invalid placeholder (substitute raises ValueError, safe_substitute keeps it)"""
import string


class CaseSensitive(string.Template):
    flags = 0


s = CaseSensitive("$wHO likes ${WHAT} for ${meal}")
d = {"wHO": "tim", "WHAT": "ham", "meal": "dinner", "w": "fred"}
# '$wHO' no longer matches the lowercase idpattern -> invalid -> ValueError.
_raised = False
try:
    s.substitute(d)
except ValueError:
    _raised = True
assert _raised, "flags=0 makes '$wHO' invalid -> ValueError"
# safe_substitute keeps the invalid parts and fills what it can.
assert s.safe_substitute(d) == "fredHO likes ${WHAT} for dinner", "flags override safe"
print("template_subclass_flags_case_sensitive OK")
