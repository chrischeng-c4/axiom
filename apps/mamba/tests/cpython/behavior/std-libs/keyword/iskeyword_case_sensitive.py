# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "iskeyword_case_sensitive"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword is case-sensitive: 'class'/'True' are keywords, 'Class'/'CLASS'/'true'/'FALSE' are not"""
import keyword

for word, expected in [
    ("class", True), ("Class", False), ("CLASS", False),
    ("True", True), ("true", False), ("FALSE", False),
]:
    assert keyword.iskeyword(word) == expected, (word, expected)

print("iskeyword_case_sensitive OK")
