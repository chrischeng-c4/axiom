# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "kwlist_matches_iskeyword"
# subject = "keyword.kwlist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.kwlist: kwlist is sorted, duplicate-free, and iskeyword(x) == (x in kwlist) for every tested value"""
import keyword

kwlist = keyword.kwlist
assert kwlist == sorted(kwlist), "kwlist must be sorted"
assert len(kwlist) == len(set(kwlist)), "kwlist must have no duplicates"

# iskeyword(x) agrees with membership in kwlist for hard keywords, ordinary
# names, and soft keywords alike.
for word in kwlist + ["user", "data", "match", "case", "type", "CLASS"]:
    assert keyword.iskeyword(word) == (word in kwlist), (
        f"iskeyword({word!r}) inconsistent with kwlist membership"
    )

print("kwlist_matches_iskeyword OK")
