# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "behavior"
# case = "context_manager_commits_on_success"
# subject = "sqlite3.Connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: using the connection as a context manager auto-commits the enclosed writes when the block exits normally"""
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute("CREATE TABLE ctx (val INTEGER)")
with conn:
    conn.execute("INSERT INTO ctx VALUES (100)")
cur = conn.cursor()
cur.execute("SELECT COUNT(*) FROM ctx")
assert cur.fetchone()[0] == 1, "context manager commits the enclosed write"
conn.close()

print("context_manager_commits_on_success OK")
