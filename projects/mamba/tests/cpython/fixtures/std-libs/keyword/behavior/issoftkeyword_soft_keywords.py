# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "issoftkeyword_soft_keywords"
# subject = "keyword.issoftkeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.issoftkeyword: issoftkeyword is True for soft keywords (match/case/type/_) and False for hard keywords"""
import keyword

for soft in ["match", "case", "type", "_"]:
    assert keyword.issoftkeyword(soft), f"{soft!r} should be a soft keyword"
for hard in ["class", "def", "return", "if", "import"]:
    assert not keyword.issoftkeyword(hard), f"{hard!r} is hard, not soft"
assert not keyword.issoftkeyword("xyzzy"), "ordinary name is not a soft keyword"

print("issoftkeyword_soft_keywords OK")
