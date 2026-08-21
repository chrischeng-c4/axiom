# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_keyword_fields"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: named/keyword fields resolve from kwargs: Formatter().format('-{arg}-', arg='test') == '-test-'"""
import string

fmt = string.Formatter()
assert fmt.format("-{arg}-", arg="test") == "-test-", "keyword field"
assert fmt.format("{first}{second}", first="a", second="b") == "ab", "two keyword fields"
print("formatter_keyword_fields OK")
