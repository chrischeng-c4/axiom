# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_symbolic_group_reference"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: \\g<name> and \\g<N> symbolic references reinsert groups; an unmatched optional group expands to the empty string"""
import re

# \g<name> reinserts a named group.
assert re.sub(r"(?P<a>x)", r"[\g<a>]", "xx") == "[x][x]", "named \\g ref"
# \g<N> reinserts a numbered group (group swap).
assert re.sub(r"(\w)(\w)", r"\g<2>\g<1>", "ab") == "ba", "numbered \\g ref"
# An unmatched optional group expands to the empty string.
assert re.sub(r"(?P<a>x)|(?P<b>y)", r"\g<b>", "xx") == "", "unmatched group -> empty"

print("sub_symbolic_group_reference OK")
