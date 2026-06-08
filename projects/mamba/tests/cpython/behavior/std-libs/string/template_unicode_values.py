# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_unicode_values"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: substitution copies arbitrary unicode/control characters from the mapping verbatim into the result"""
import string

s = string.Template("$who likes $what")
d = {"who": "t\xffm", "what": "f\xfe\x0ced"}
assert s.substitute(d) == "t\xffm likes f\xfe\x0ced", "unicode/control values copied verbatim"
print("template_unicode_values OK")
