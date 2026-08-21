# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "star_matches_any_run"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatchcase: * matches any run of characters including zero and the empty string; '*.txt' matches multi-segment names and a leading dot, and bare '*' matches the empty string"""
import fnmatch

# * spans any run of characters, including across dots.
assert fnmatch.fnmatchcase("hello.world.txt", "*.txt"), "* multi-segment"
# '*.txt' still requires the literal dot.
assert fnmatch.fnmatchcase("txt", "*.txt") is False, "*.txt requires dot"
assert fnmatch.fnmatchcase(".txt", "*.txt"), "*.txt matches .txt"
# * matches the empty string and a leading-dot name.
assert fnmatch.fnmatchcase(".hidden", "*"), "* matches leading dot"
assert fnmatch.fnmatchcase("", "*"), "* matches empty"

print("star_matches_any_run OK")
