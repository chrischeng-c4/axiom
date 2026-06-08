# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "char_class_set_membership"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: [abc] matches set membership: '[abc]at' matches 'cat' but not 'xat'; the negated '[!abc]at' matches 'xat'"""
import fnmatch

assert fnmatch.fnmatchcase("cat", "[abc]at"), "set member matches"
assert not fnmatch.fnmatchcase("xat", "[abc]at"), "non-member excluded"
assert fnmatch.fnmatchcase("xat", "[!abc]at"), "negated set matches non-member"

print("char_class_set_membership OK")
