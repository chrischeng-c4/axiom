# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "named_parameter_binding"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: ':name' named placeholders bind a dict of values into INSERT/SELECT so the round-tripped value matches"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE items (name TEXT, value INTEGER)")
cur.execute("INSERT INTO items VALUES (:name, :value)", {"name": "named", "value": 99})
cur.execute("SELECT value FROM items WHERE name = :n", {"n": "named"})
v = cur.fetchone()[0]
assert v == 99, f"named parameter = {v!r}"
conn.close()

print("named_parameter_binding OK")
