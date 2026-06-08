# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_braced_substitution"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: ${braced} fields delimit adjacent text: Template('${prefix}ing').substitute(prefix='walk') == 'walking' and mixed $who/${what} forms resolve"""
import string

assert string.Template("${prefix}ing").substitute(prefix="walk") == "walking", "braced sub"
s = string.Template("$who likes ${what} for ${meal}")
d = {"who": "tim", "what": "ham", "meal": "dinner"}
assert s.substitute(d) == "tim likes ham for dinner", "braced fields"
print("template_braced_substitution OK")
