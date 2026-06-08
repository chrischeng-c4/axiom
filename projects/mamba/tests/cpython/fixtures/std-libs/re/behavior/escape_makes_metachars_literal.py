# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "escape_makes_metachars_literal"
# subject = "re.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.escape: re.escape turns metacharacters into literals: the escaped 'a.b+c?' matches that exact text but the escaped dot does NOT match an arbitrary char"""
import re

escaped = re.escape("a.b+c?")
assert re.search(escaped, "a.b+c?") is not None, "escaped text matches itself"
assert re.search(escaped, "axb+c?") is None, "escaped dot does NOT match any char"

print("escape_makes_metachars_literal OK")
