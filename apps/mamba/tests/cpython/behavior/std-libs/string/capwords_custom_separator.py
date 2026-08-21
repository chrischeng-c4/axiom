# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "capwords_custom_separator"
# subject = "string.capwords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.capwords: a custom separator splits and re-joins on that separator without collapsing: 'ABC-DEF-GHI', '-' -> 'Abc-Def-Ghi' and a tab separator is preserved literally"""
import string

assert string.capwords("hello/world", "/") == "Hello/World", "capwords slash separator"
assert string.capwords("ABC-DEF-GHI", "-") == "Abc-Def-Ghi", "capwords dash separator"
assert string.capwords("ABC-def DEF-ghi GHI", "-") == "Abc-Def def-Ghi ghi", "capwords dash splits only on dash"
assert string.capwords("\taBc\tDeF\t", "\t") == "\tAbc\tDef\t", "capwords tab separator literal"
print("capwords_custom_separator OK")
