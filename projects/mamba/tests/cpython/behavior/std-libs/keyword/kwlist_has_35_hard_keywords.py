# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "kwlist_has_35_hard_keywords"
# subject = "keyword.kwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.kwlist: kwlist contains exactly 35 hard keywords on CPython 3.12, including async/await (PEP 492)"""
import keyword

assert len(keyword.kwlist) == 35, f"expected 35 hard keywords, got {len(keyword.kwlist)}"
assert "async" in keyword.kwlist, "async (PEP 492) must be a hard keyword"
assert "await" in keyword.kwlist, "await (PEP 492) must be a hard keyword"

print("kwlist_has_35_hard_keywords OK")
