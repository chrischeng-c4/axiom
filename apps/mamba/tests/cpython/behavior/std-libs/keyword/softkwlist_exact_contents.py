# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "softkwlist_exact_contents"
# subject = "keyword.softkwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.softkwlist: softkwlist is exactly ['_', 'case', 'match', 'type'] (sorted) on CPython 3.12 (PEP 695 'type')"""
import keyword

assert keyword.softkwlist == ["_", "case", "match", "type"], (
    f"unexpected softkwlist={keyword.softkwlist!r}"
)
assert keyword.softkwlist == sorted(keyword.softkwlist), "softkwlist must be sorted"

print("softkwlist_exact_contents OK")
