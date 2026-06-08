# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "register_get_unregister_dialect_round_trip"
# subject = "csv.register_dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.register_dialect: a registered named dialect is usable by reader/writer and disappears after unregister_dialect"""
import csv
import io

csv.register_dialect("pipes", delimiter="|", quoting=csv.QUOTE_MINIMAL)
assert "pipes" in csv.list_dialects(), "pipes not registered"
assert csv.get_dialect("pipes").delimiter == "|", "pipes delimiter"

buf = io.StringIO()
csv.writer(buf, dialect="pipes").writerow(["x", "y", "z"])
assert buf.getvalue() == "x|y|z\r\n", f"pipes write = {buf.getvalue()!r}"

rows = list(csv.reader(io.StringIO("x|y|z"), dialect="pipes"))
assert rows == [["x", "y", "z"]], f"pipes read = {rows!r}"

csv.unregister_dialect("pipes")
assert "pipes" not in csv.list_dialects(), "pipes still registered"

print("register_get_unregister_dialect_round_trip OK")
