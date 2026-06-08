# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_positional_fields"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: Formatter.format passes plain text through and fills explicit positional fields, reusing an index across the template: 'foo{1}{0}-{1}' with ('bar', 6) -> 'foo6bar-6'"""
import string

fmt = string.Formatter()
assert fmt.format("foo") == "foo", "plain text"
assert fmt.format("foo{0}", "bar") == "foobar", "positional 0"
assert fmt.format("foo{1}{0}-{1}", "bar", 6) == "foo6bar-6", "reused positional"
print("formatter_positional_fields OK")
