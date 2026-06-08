# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "primary_key_violation_raises_integrityerror"
# subject = "sqlite3.Cursor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: inserting a duplicate PRIMARY KEY value raises sqlite3.IntegrityError (a DatabaseError subclass)"""
import sqlite3

conn = sqlite3.connect(":memory:")
cur = conn.cursor()
cur.execute("CREATE TABLE pk_test (id INTEGER PRIMARY KEY, name TEXT)")
cur.execute("INSERT INTO pk_test VALUES (1, 'first')")
_raised = False
try:
    cur.execute("INSERT INTO pk_test VALUES (1, 'duplicate')")
except sqlite3.IntegrityError:
    _raised = True
assert _raised, "duplicate primary key raises IntegrityError"
conn.close()

print("primary_key_violation_raises_integrityerror OK")
