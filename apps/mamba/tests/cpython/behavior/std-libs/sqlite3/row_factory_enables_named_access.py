# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "row_factory_enables_named_access"
# subject = "sqlite3.Row"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Row: setting connection.row_factory = sqlite3.Row lets fetched rows be indexed by column name as well as by position"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
conn.execute("INSERT INTO users VALUES (1, 'Alice', 30)")
conn.commit()
conn.row_factory = sqlite3.Row
cur = conn.cursor()
cur.execute("SELECT * FROM users WHERE id = 1")
row = cur.fetchone()
assert row["name"] == "Alice", f"Row by name = {row['name']!r}"
assert row["age"] == 30, f"Row by name = {row['age']!r}"
assert row[1] == "Alice", f"Row by position = {row[1]!r}"
conn.close()

print("row_factory_enables_named_access OK")
