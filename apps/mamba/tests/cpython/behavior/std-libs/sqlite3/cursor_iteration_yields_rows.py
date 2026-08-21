# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "cursor_iteration_yields_rows"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: iterating a cursor after execute() yields each result row in order, one at a time"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE iter_t (x INTEGER)")
conn.executemany("INSERT INTO iter_t VALUES (?)", [(i,) for i in range(5)])
conn.commit()
cur = conn.cursor()
cur.execute("SELECT x FROM iter_t ORDER BY x")
vals = [row[0] for row in cur]
assert vals == [0, 1, 2, 3, 4], f"cursor iter = {vals!r}"
conn.close()

print("cursor_iteration_yields_rows OK")
