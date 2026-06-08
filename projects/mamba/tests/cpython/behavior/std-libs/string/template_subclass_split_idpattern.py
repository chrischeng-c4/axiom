# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "template_subclass_split_idpattern"
# subject = "string.Template"
# kind = "semantic"
# xfail = "string.Template subclassing relies on the substitution engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Template: a Template subclass with separate idpattern (lower) and braceidpattern (upper) honors the split grammar: '$foo ${BAR}' resolves but '$FOO' and '${bar}' are invalid placeholders"""
import string


class SplitPattern(string.Template):
    idpattern = "[a-z]+"
    braceidpattern = "[A-Z]+"
    flags = 0


m = {"foo": "foo", "BAR": "BAR"}
assert SplitPattern("$foo ${BAR}").substitute(m) == "foo BAR", "split id/brace patterns"
# Unbraced upper-case and braced lower-case violate the split grammar.
for text in ("$FOO", "${bar}"):
    _raised = False
    try:
        SplitPattern(text).substitute(m)
    except ValueError:
        _raised = True
    assert _raised, f"expected ValueError for {text!r}"
print("template_subclass_split_idpattern OK")
