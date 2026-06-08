# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "null_roundtrips_as_none"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: SQL NULL values are stored and retrieved as Python None for both TEXT and INTEGER columns"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE nulls (a TEXT, b INTEGER)")
conn.execute("INSERT INTO nulls VALUES (NULL, NULL)")
conn.commit()
row = conn.execute("SELECT * FROM nulls").fetchone()
assert row[0] is None, f"NULL TEXT = {row[0]!r}"
assert row[1] is None, f"NULL INT = {row[1]!r}"
conn.close()

print("null_roundtrips_as_none OK")
