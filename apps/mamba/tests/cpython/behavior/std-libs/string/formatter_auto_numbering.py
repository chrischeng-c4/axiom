# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_auto_numbering"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: automatic field numbering ({} == {0},{1},...) fills in argument order and supports an auto-numbered nested width: '{:^{}}' with ('bar', 6) -> ' bar  '"""
import string

fmt = string.Formatter()
assert fmt.format("foo{}{}", "bar", 6) == "foobar6", "auto numbering"
assert fmt.format("{:^{}}", "bar", 6) == " bar  ", "auto-numbered nested width"
print("formatter_auto_numbering OK")
