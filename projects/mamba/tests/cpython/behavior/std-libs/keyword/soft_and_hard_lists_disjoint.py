# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "soft_and_hard_lists_disjoint"
# subject = "keyword.softkwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.softkwlist: kwlist and softkwlist are disjoint; soft keywords are absent from kwlist and bind as ordinary names via exec"""
import keyword

# The two lists never overlap.
for kw in keyword.kwlist:
    assert kw not in keyword.softkwlist, f"{kw!r} in both lists"
for sk in keyword.softkwlist:
    assert sk not in keyword.kwlist, f"{sk!r} in both lists"

# Every soft keyword binds as an ordinary identifier at runtime via exec.
ns = {}
for sk in keyword.softkwlist:
    exec(f"{sk} = 42", ns)
    assert ns[sk] == 42, f"soft keyword {sk!r} should bind to 42"

print("soft_and_hard_lists_disjoint OK")
