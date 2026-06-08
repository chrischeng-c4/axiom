# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "executemany_inserts_each_row"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: executemany(sql, seq) runs the statement once per parameter tuple; the row count reflects every inserted tuple"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
cur.executemany("INSERT INTO users VALUES (?, ?, ?)", [
    (10, "Dave", 40),
    (11, "Eve", 35),
])
cur.execute("SELECT COUNT(*) FROM users")
cnt = cur.fetchone()[0]
assert cnt == 2, f"count after executemany = {cnt!r}"
conn.close()

print("executemany_inserts_each_row OK")
