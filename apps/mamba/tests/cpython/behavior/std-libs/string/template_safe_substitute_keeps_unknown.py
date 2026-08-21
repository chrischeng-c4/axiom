# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_safe_substitute_keeps_unknown"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .safe_substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: safe_substitute fills known fields and leaves unknown placeholders literally in place instead of raising: 'Hello $name, $greeting!' with name only keeps '$greeting'"""
import string

t = string.Template("Hello $name, $greeting!")
result = t.safe_substitute(name="Alice")
assert "Alice" in result, f"safe name = {result!r}"
assert "$greeting" in result, f"safe missing kept = {result!r}"
# A braced-form unknown is kept verbatim too.
t2 = string.Template("$known $unknown")
s2 = t2.safe_substitute(known="hi")
assert "hi" in s2 and "$unknown" in s2, f"safe_substitute = {s2!r}"
print("template_safe_substitute_keeps_unknown OK")
