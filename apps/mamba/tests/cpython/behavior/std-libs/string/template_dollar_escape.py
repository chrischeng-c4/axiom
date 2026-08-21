# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_dollar_escape"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a doubled $$ is a literal '$': Template('Cost: $$100').substitute() == 'Cost: $100' and a mixed '$$100' escape inside named fields stays literal"""
import string

assert string.Template("Cost: $$100").substitute() == "Cost: $100", "literal $ escape"
s = string.Template("$who likes to eat a bag of $what worth $$100")
got = s.substitute({"who": "tim", "what": "ham"})
assert got == "tim likes to eat a bag of ham worth $100", f"mixed = {got!r}"
print("template_dollar_escape OK")
