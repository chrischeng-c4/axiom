# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "qmark_parameter_binding"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: '?' positional placeholders bind a tuple of values into INSERT/SELECT so the round-tripped value matches"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE items (name TEXT, value INTEGER)")
cur.execute("INSERT INTO items VALUES (?, ?)", ("test", 42))
cur.execute("SELECT value FROM items WHERE name = ?", ("test",))
v = cur.fetchone()[0]
assert v == 42, f"parameterized = {v!r}"
conn.close()

print("qmark_parameter_binding OK")
