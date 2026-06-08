# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "join_trailing_slash_preserved"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join keeps an empty/trailing-slash final component; join('a','') == 'a/' and join('a','b/') == 'a/b/' while join('a/','b') == 'a/b'"""
import os.path

assert os.path.join("a", "") == "a/", "empty final segment"
assert os.path.join("a", "b/") == "a/b/", "trailing slash preserved"
assert os.path.join("a/", "b") == "a/b", "leading slash in first segment"

print("join_trailing_slash_preserved OK")
