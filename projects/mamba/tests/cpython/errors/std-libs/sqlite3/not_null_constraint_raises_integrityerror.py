# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "not_null_constraint_raises_integrityerror"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: inserting NULL into a NOT NULL column raises sqlite3.IntegrityError"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE u (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
_raised = False
try:
    cur.execute("INSERT INTO u (id, name) VALUES (?, ?)", (1, None))
except sqlite3.IntegrityError:
    _raised = True
assert _raised, "NULL into NOT NULL column raises IntegrityError"
conn.close()

print("not_null_constraint_raises_integrityerror OK")
