# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_identifier_grammar"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template is a silent dict-stub on mamba; .substitute() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: identifiers allow digits/underscores and are case-sensitive: '$_wh0_ ${_w_h_a_t_} ${mea1}' and upper-case '$WHO ${WHAT}' both resolve from their mappings"""
import string

# Identifiers may contain digits and underscores.
s = string.Template("$_wh0_ likes ${_w_h_a_t_} for ${mea1}")
d = {"_wh0_": "tim", "_w_h_a_t_": "ham", "mea1": "dinner"}
assert s.substitute(d) == "tim likes ham for dinner", "non-letter identifiers"
# Identifiers are case-sensitive and may be upper-case.
s = string.Template("$WHO likes ${WHAT} for ${MEAL}")
d = {"WHO": "tim", "WHAT": "ham", "MEAL": "dinner"}
assert s.substitute(d) == "tim likes ham for dinner", "upper-case identifiers"
print("template_identifier_grammar OK")
