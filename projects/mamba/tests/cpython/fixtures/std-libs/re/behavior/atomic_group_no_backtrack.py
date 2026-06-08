# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "atomic_group_no_backtrack"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: an atomic group (?>bc|b) takes its match and refuses to give it back: r'a(?>bc|b)c' rejects 'abc' but accepts 'abcc'"""
import re

pat = re.compile(r"a(?>bc|b)c")
assert pat.match("abc") is None, "atomic refuses to backtrack to b"
assert pat.match("abcc") is not None, "atomic matches bc then c"
assert re.match(r"(?>.*).", "abc") is None, "atomic .* leaves nothing"

print("atomic_group_no_backtrack OK")
