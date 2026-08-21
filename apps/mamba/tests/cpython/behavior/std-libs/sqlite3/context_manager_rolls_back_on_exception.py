# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "context_manager_rolls_back_on_exception"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: when an exception escapes the connection context-manager block, the enclosed writes are rolled back and the exception re-raised"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE ctx (val INTEGER)")
_raised = False
try:
    with conn:
        conn.execute("INSERT INTO ctx VALUES (200)")
        raise ValueError("force rollback")
except ValueError:
    _raised = True
assert _raised, "the escaping ValueError is re-raised"
cur = conn.cursor()
cur.execute("SELECT COUNT(*) FROM ctx")
assert cur.fetchone()[0] == 0, "the enclosed write was rolled back"
conn.close()

print("context_manager_rolls_back_on_exception OK")
