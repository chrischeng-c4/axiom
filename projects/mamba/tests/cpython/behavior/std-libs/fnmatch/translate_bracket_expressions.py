# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_bracket_expressions"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate bracket rules: [abc]->[abc], []]->[]] (literal close-bracket first), [!x]->[^x], [^x]->[\\^x] (literal caret escaped), [x->\\[x (unterminated bracket is literal)"""
import fnmatch

assert fnmatch.translate("[abc]") == "(?s:[abc])\\Z", "class"
assert fnmatch.translate("[]]") == "(?s:[]])\\Z", "literal close-bracket first"
assert fnmatch.translate("[!x]") == "(?s:[^x])\\Z", "negated class -> caret"
assert fnmatch.translate("[^x]") == "(?s:[\\^x])\\Z", "literal caret escaped"
assert fnmatch.translate("[x") == "(?s:\\[x)\\Z", "unterminated bracket is literal"

print("translate_bracket_expressions OK")
