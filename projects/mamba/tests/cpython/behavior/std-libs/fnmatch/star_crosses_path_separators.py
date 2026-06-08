# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "star_crosses_path_separators"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatchcase: fnmatch is string-level not path-level: * spans '/' so 'a/b/c.txt' matches '*.txt' and '*/*.txt'"""
import fnmatch

# Unlike glob, fnmatch is string-level — * matches the '/' separator too.
assert fnmatch.fnmatchcase("a/b/c.txt", "*.txt"), "* crosses path separators"
assert fnmatch.fnmatchcase("a/b/c.txt", "*/*.txt"), "nested glob across separators"

print("star_crosses_path_separators OK")
