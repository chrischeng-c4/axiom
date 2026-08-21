# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_expand_template"
# subject = "re.Match.expand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.expand: Match.expand fills a template from the captured groups by number (\\1) and by name (\\g<name>)"""
import re

m = re.match(r"(?P<first>\w+) (?P<second>\w+)", "hello world")
assert m.expand(r"\2 \1") == "world hello", "expand by number"
assert m.expand(r"\g<second>-\g<first>") == "world-hello", "expand by name"

print("match_expand_template OK")
