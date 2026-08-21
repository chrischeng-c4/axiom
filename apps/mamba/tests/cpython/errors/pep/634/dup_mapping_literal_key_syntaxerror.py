# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "dup_mapping_literal_key_syntaxerror"
# subject = "match.mapping_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.mapping_pattern: dup_mapping_literal_key_syntaxerror (errors)."""
pass

_raised = False
try:
    compile("match {}:\n case {'a': 1, 'a': 2}: pass", '<dup>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "dup_mapping_literal_key_syntaxerror: expected SyntaxError"
print("dup_mapping_literal_key_syntaxerror OK")
