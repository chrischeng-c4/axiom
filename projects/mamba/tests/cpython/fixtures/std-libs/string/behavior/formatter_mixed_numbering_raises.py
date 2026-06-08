# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_mixed_numbering_raises"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter is a silent dict-stub on mamba; .format() AttributeErrors (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: mixing automatic ({}) and manual ({1}) field numbering in one template raises ValueError, for both 'foo{1}{}' and 'foo{}{1}'"""
import string

fmt = string.Formatter()
for bad in ("foo{1}{}", "foo{}{1}"):
    _raised = False
    try:
        fmt.format(bad, "bar", 6)
    except ValueError:
        _raised = True
    assert _raised, f"expected ValueError for {bad!r}"
print("formatter_mixed_numbering_raises OK")
