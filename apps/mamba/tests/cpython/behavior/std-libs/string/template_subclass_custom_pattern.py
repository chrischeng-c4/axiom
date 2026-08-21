# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_subclass_custom_pattern"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template subclassing relies on the substitution engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a Template subclass overriding `pattern` with a custom @@braced@@ group substitutes through the new grammar and safe_substitute keeps unresolved placeholders intact"""
import string


class MyTemplate(string.Template):
    pattern = r"""
        \$(?:
          (?P<escaped>\$)                    |
          (?P<named>[_a-z][_a-z0-9]*)        |
          @@(?P<braced>[_a-z][_a-z0-9]*)@@   |
          (?P<invalid>)                      |
        )
        """


tmpl = "PyCon in $@@location@@"
t = MyTemplate(tmpl)
_raised = False
try:
    t.substitute({})
except KeyError:
    _raised = True
assert _raised, "custom pattern missing key raises KeyError"
assert t.substitute({"location": "Cleveland"}) == "PyCon in Cleveland", "custom pattern substitute"
assert t.safe_substitute() == tmpl, "custom pattern safe_substitute keeps text"
assert t.safe_substitute({"location": "Cleveland"}) == "PyCon in Cleveland", "custom pattern safe_substitute"
print("template_subclass_custom_pattern OK")
