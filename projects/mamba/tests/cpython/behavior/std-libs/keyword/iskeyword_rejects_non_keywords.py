# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "iskeyword_rejects_non_keywords"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword returns False for ordinary identifiers, empty string, and soft keywords"""
import keyword

non_keywords = ["user", "data", "value", "hello", "match_", "", "match", "case", "type"]
for word in non_keywords:
    assert not keyword.iskeyword(word), f"{word!r} should not be a keyword"

print("iskeyword_rejects_non_keywords OK")
