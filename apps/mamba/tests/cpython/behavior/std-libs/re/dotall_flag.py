# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "dotall_flag"
# subject = "re.DOTALL"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.DOTALL: by default '.' does not match newline; re.DOTALL makes 'a.b' match 'a\\nb'"""
import re

assert re.search(r"a.b", "a\nb") is None, "dot does not match newline by default"
assert re.search(r"a.b", "a\nb", re.DOTALL) is not None, "DOTALL -> dot matches newline"

print("dotall_flag OK")
