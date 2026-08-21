# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dict_field_order_preserved_round_trip"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: fieldname order survives a DictWriter header -> DictReader round trip across several orderings"""
import csv
import io

for keys in (["a", "b", "c"], ["c", "a", "b"], ["b", "c", "a"]):
    buf = io.StringIO()
    csv.DictWriter(buf, keys).writeheader()
    buf.seek(0)
    assert csv.DictReader(buf).fieldnames == keys, f"order {keys!r}"

print("dict_field_order_preserved_round_trip OK")
