# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "custom_quotechar_used_for_quoting"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: a custom quotechar replaces the default double-quote when QUOTE_ALL wraps fields"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, quotechar="'", quoting=csv.QUOTE_ALL).writerow(["hello", "world"])
out = buf.getvalue().strip()
assert out == "'hello','world'", f"custom quotechar = {out!r}"

print("custom_quotechar_used_for_quoting OK")
