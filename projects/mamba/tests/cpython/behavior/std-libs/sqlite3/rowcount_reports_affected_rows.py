# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "rowcount_reports_affected_rows"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: cursor.rowcount reports the number of rows affected by an UPDATE"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, age INTEGER)")
cur.execute("INSERT INTO users VALUES (1, 30)")
cur.execute("UPDATE users SET age = 31 WHERE id = 1")
assert cur.rowcount == 1, f"rowcount = {cur.rowcount!r}"
conn.close()

print("rowcount_reports_affected_rows OK")
