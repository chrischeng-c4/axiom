# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_index_and_attribute_lookup"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: {0[i]} indexes into a sequence argument and {0.attr} reads an attribute: '{0[2]}{0[0]}' over ['eggs','and','spam'] -> 'spameggs'"""
import string

fmt = string.Formatter()
assert fmt.format("{0[2]}{0[0]}", ["eggs", "and", "spam"]) == "spameggs", "index lookup"


class AnyAttr:
    def __getattr__(self, attr):
        return attr


assert fmt.format("{0.lumber}{0.jack}", AnyAttr()) == "lumberjack", "attr lookup"
print("formatter_index_and_attribute_lookup OK")
