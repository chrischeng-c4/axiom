# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "named_groups_and_groupdict"
# subject = "re.Match.groupdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.groupdict: named groups resolve by name and groupdict() returns the name->text map: r'(?P<year>\\d{4})-(?P<month>\\d{2})' on '2024-03'"""
import re

m = re.search(r"(?P<year>\d{4})-(?P<month>\d{2})", "date: 2024-03")
assert m is not None, "named groups match"
assert m.group("year") == "2024", f"year = {m.group('year')!r}"
assert m.group("month") == "03", f"month = {m.group('month')!r}"
assert m.groupdict() == {"year": "2024", "month": "03"}, f"groupdict = {m.groupdict()!r}"

print("named_groups_and_groupdict OK")
