# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_named_substitution"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: $name fields are replaced from keyword args and the mapping: Template('$first $last').substitute(first='John', last='Doe') == 'John Doe'"""
import string

# Keyword form.
t = string.Template("$first $last")
assert t.substitute(first="John", last="Doe") == "John Doe", "template kwargs sub"
# Mapping form, and a numeric example.
assert string.Template("$x + $y = $z").substitute({"x": 1, "y": 2, "z": 3}) == "1 + 2 = 3", "template mapping sub"
print("template_named_substitution OK")
