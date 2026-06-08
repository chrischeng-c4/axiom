# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "lastrowid_reports_inserted_rowid"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: cursor.lastrowid reports the rowid of the most recently inserted row"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
cur.execute("INSERT INTO users VALUES (3, 'Carol')")
assert cur.lastrowid == 3, f"lastrowid = {cur.lastrowid!r}"
conn.close()

print("lastrowid_reports_inserted_rowid OK")
