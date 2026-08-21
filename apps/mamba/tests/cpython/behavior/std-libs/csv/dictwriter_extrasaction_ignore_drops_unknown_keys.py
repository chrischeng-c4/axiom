# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictwriter_extrasaction_ignore_drops_unknown_keys"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: extrasaction='ignore' silently drops keys not in fieldnames and writes only the known fields"""
import csv
import io

buf = io.StringIO()
dw = csv.DictWriter(buf, ["f1", "f2"], extrasaction="ignore")
dw.writerow({"f0": 0, "f1": 1, "f2": 2, "f3": 3})
assert buf.getvalue() == "1,2\r\n", f"ignore = {buf.getvalue()!r}"

print("dictwriter_extrasaction_ignore_drops_unknown_keys OK")
