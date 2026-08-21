# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_conversion_specifiers"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: the !r/!s/!a conversions apply repr/str/ascii: '{0!a}' escapes chr(255) as '\\xff' and chr(256) as '\\u0100', '{arg!r}' quotes a str"""
import string

fmt = string.Formatter()
assert fmt.format("-{arg!r}-", arg="test") == "-'test'-", "!r conversion"
assert fmt.format("{0!s}", "test") == "test", "!s conversion"
assert fmt.format("{0!a}", chr(255)) == "'\\xff'", "!a escapes non-ascii"
assert fmt.format("{0!a}", chr(256)) == "'\\u0100'", "!a escapes wide"
print("formatter_conversion_specifiers OK")
